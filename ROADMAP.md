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

Full thesis: [docs/architecture.md](docs/architecture.md)
Compiler laws and coercion model: [docs/compiler-laws.md](docs/compiler-laws.md)
Coercion design (algebra-keyed inhabitants): [docs/coercion-design.md](docs/coercion-design.md)
Testing strategy: [docs/testing-strategy.md](docs/testing-strategy.md)
Invariant enforcement: [INVARIANTS.md](INVARIANTS.md)
Modeling guidelines: [MODELING.md](MODELING.md)

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

See [docs/cx-design.md](docs/cx-design.md) for the full diagnosis,
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

Implementation plan: [docs/cx-design.md §Option B](docs/cx-design.md).

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
| **C: Complexity** | complexity.dag, docs/cx-*, docs/cost-* | 04_*, 05_*, dsl/, tests/ |
| **D: DSL Modeling** | dsl/std/, dsl/extdeps/{llm,github,shell,cron,cloud,git}/, dsl/gunbc/, dsl/tools/, dsl/config/ | src/v2/*.dag (compiler sources) |
| **E: Testing** | src/v2/tests/, scripts/, docs/testing-*, std/verification.dag, compiler_tests_rust.dag, coercion.dag (test extraction) | 04_*, 05_emit*, complexity |

---

## CI gates

| Gate | Command | Status |
|------|---------|--------|
| Lint | `cargo clippy --workspace -- -D warnings` | GREEN |
| Tests | `cargo test -p v2-compiler-tests` | GREEN (388 pass) |
| Full DSL | `full_dsl_compiles -- --ignored` | GREEN |
| Diagnostic ratchet | `strict_compile_diagnostic_count -- --ignored` | 424 (honest, non-blocking) |
| L1 gate | `scripts/l1-ratchet.sh --check` | GREEN (0, hard gate) |
| Stage0 freshness | `scripts/check-stage0-freshness.sh` | GREEN (blocking) |

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
C1 (direct SubValueRelation→LoweringTarget) and S8 (lattice
inhabitant declarations). Both small, both unblocked. C1 enables
better bounds when C2 lands. S8 dissolves ad-hoc merge functions.

### What to start now

| Stream | Items | Files |
|--------|-------|-------|
| A: Provenance infra | S1, S2, S3, S4, S5 | 04_env.dag, 04_infer.dag |
| B: Clone elision | O1-O5 | ownership.dag, 05_emit_rust.dag |
| C: std/ foundation | C1, ~~S8~~ (DONE) | std/induction.dag, std/algebra.dag, std/termination.dag |

Zero file overlap between streams. After Stream A (S1-S5), CX
consumer items C2-C6 can start (same files as Stream A, sequential).

### Active workboards

- **CX:** [docs/cx-design.md §Workboard](docs/cx-design.md)
  — S1-S8 shared, C1-C6 CX-specific, TDD plan, cleanup catalog
- **Ownership:** [docs/ownership-design.md §Workboard](docs/ownership-design.md)
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
| LS-4: Ownership/borrowing | See [ownership-design.md](docs/ownership-design.md) |
| LS-5: Visibility/module system | Not started |
| LS-6: Shared typed handlers | DONE (PR #355) |

**LS-4: Ownership — three layers to clone elimination**

Stage0 emits 23,733 `.clone()` calls (~0.479 clones/line). The
ownership analysis (PR #313) already computes the facts needed to
eliminate most of them. The gap: the emitter doesn't consume all
the facts it has.

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
| 1. Last-use elision | 1-2 PRs | Nothing | ~2,000-4,000 clones |
| 2. Post-TCO ownership | 1 PR | Nothing | ~500-1,000 clones |
| 3. Borrow propagation | 3-5 PRs | LS-4 design | ~15,000-18,000 clones |

**Layer 1 (last-use elision):** For each binding with fan-out > 1,
the last use site moves instead of cloning. The data exists in
BindingUsage — the emitter just needs to track which use is last
and skip the `.clone()`. Unblocked.

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
| CG-2: Expression-level gaps (TLC-1..4) | TLC-1/2/3 done, TLC-4 partial |
| CG-3: Parameterization (3 backends → 1 homomorphism) | Phases 1-3 done, 4-6 deferred |

### Track 5: Real-program emission (Lane B + D)

**Goal:** Compile real .dag programs to fully executable Rust.
First target: `gunbc/tools/review.dag` (PR review agent).

| Item | Status |
|------|--------|
| RE-1: Transport emission fidelity (REST, shell) | DONE (21/21) |
| RE-2: review.dag compiles (dry-run) | DONE |
| RE-3: review.dag passes live integration | Partial |
| RE-4: Anthropic REST API end-to-end | Not started |

### Track 6: Algebra field dispatch (Lane A)

**Known debt (M8/M9).** `ExprBinOp.algebra_field: String?` dispatches
via string values ("add", "mul", etc.). Replace with structural
`AlgebraFieldKind` coproduct.

Status: Documented, not started. Low urgency (few consumers), but
exemplifies the string-dispatch anti-pattern.

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
- `emit_rust_default_value` (05_emit_rust.dag) is a hand-written
  if-forest over type names. `TypeCheckpoint.default_expr` already
  carries the same defaults. Fix: read from checkpoint data.
- `rust_type_map` (extdeps/languages/rust/emit.dag) duplicates the
  primitive mapping in `rust_type_checkpoints`. Same for Go/Python.
  Fix: derive from TypeCheckpoint only.
- `go_source_extension` in extdeps/languages/go/emit.dag duplicates
  `go_scaffold.source_file_extension` in std/languages.dag.
- `keyword_to_name` in 02_parse.dag duplicates the tokenizer keyword
  table. Reconcile to single authority.
- CallableOf coverage incomplete: `filter`, `any`, `all`, `sort_by`
  still omit CallableOf from their param_types in algebra templates.
  Completing this dissolves downstream string dispatch in emit + CX.
  Ignored tests in compiler_tests_rust.dag (wrong-callback-arity,
  wrong-return-type) are blocked on this.

### Track 8: Lattice inhabitant consolidation (Lane D)

`std/algebra.dag` defines `Lattice<T>` and `BoundedLattice<T>` as
types, but no concrete type declares that it inhabits them.

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
| Encoding lattice | `dsl/std/encoding.dag` names the lattice but delegates to Rust's `ContentEncoding` | Model join/meet in .dag; reconcile `Encoding` and `ContentEncoding` to one authority |
| Stack<T> → FreeMonoid | `dsl/std/stack.dag` defines bespoke push/pop/fold_stack instead of attaching to FreeMonoid | Import algebra.dag; Stack IS FreeMonoid |
| User-defined generic emission | Generic functions (T, V, K params) parse and type-check but emit unresolved type variables in Rust | Emitter needs monomorphization or generic Rust output |

### Track 10: Extdeps modeling fidelity (Lane D)

Stringly-typed fields that should be structural, surfaced by external
audit (2026-04-10):

| Item | File | Fix |
|------|------|-----|
| `GitHubAuthToken.scopes: List<String>` | extdeps/github/github.dag | Use existing `GitHubScope` enum in same file |
| `ThinkingConfig.type: String` | extdeps/llm/anthropic.dag | Structural coproduct |
| `LlmMessage.content: String` | extdeps/llm/llm.dag | Richer multimodal block structure (M1) |
| `Gist.files: List<GistFile>` | extdeps/github/gists.dag | `Map<String, GistFile>` per API shape |
| OpenAI string-path extraction | extdeps/llm/openai.dag | Structural field access (M8) |
| Policy defaults in `CloudSecretConfig` | std/types.dag | Move `"latest"`, `"sigstore"`, `Bearer` to call sites |
| `ProjectId` vs `GcpProjectId` | std/types.dag | Reconcile to one branded type |

---

## Bootstrap

**Status: COMPLETE.** Stage0 content is generated from .dag source.
One non-generated file remains: `compiler_tests.rs` (test harness,
`#[cfg(test)]` only — not part of the compiler binary). Regenerated
binary produces identical output when it self-compiles (fixed-point
convergence).

```
.dag source ──(v2-compiler)──▶ stage0 .rs ──(cargo/rustc)──▶ v2-compiler binary
     ▲                                                              │
     └──────────────────────────────────────────────────────────────┘
```

Source of truth: `.dag` files. Stage0 `.rs` is a derived artifact.

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

### KF-3: Automated test generation from types

Compiler generates tests from type definitions. Add a type → tests
appear. Grounded in finite, enumerable type algebra with canonical
witness generation.

**Status:** Not built. Design in ROADMAP history / docs.

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

### Gate 1: Model is closed

| Criterion | Test |
|-----------|------|
| Provenance on bindings (Track 1 complete) | 0 CX violations, reconstruction code deleted |
| Language specs modeled (Track 2 complete) | No inline target-language knowledge in emitter |
| Node.name deleted (Track 3 complete) | l1-ratchet = 0, field deleted |
| Codegen from structural authority (Track 4) | CG acceptance criteria met |
| RE ratchet (Track 5) | 21/21 |
| Performance | No test >2s, self-compile <30s |

### Gate 2: Killer features ship

| Criterion | Test |
|-----------|------|
| KF-1: Complexity proof | 0 CostUnknown, gate blocking |
| KF-2: Reject suboptimal | Equivalence catalog with ≥5 rules |
| KF-4: Cross-language parity | All 3 backends compile full DSL |
| KF-5: Decidable language | Working (already met) |

### Gate 3: Demo quality

| Criterion | Test |
|-----------|------|
| One impressive demo | Compile .dag agent → show Rust → run live → show complexity proof |
| Documentation | README, getting-started, language reference |
| Clean install | `cargo install gunbc` works |

### Release dependency chain

```
Track 1 (provenance) ──→ KF-1 (complexity proof) ──→ Gate 2
Track 2 (language specs) ──→ Gate 1
Track 3 (Node.name) ──→ Gate 1
Track 4 (codegen) ──→ Gate 1
Track 5 (RE) ──→ Gate 1
                                              Gate 1 + Gate 2 ──→ Gate 3 ──→ Release
```
