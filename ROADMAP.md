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

Extend TypeBinding with structural provenance — a value's relationship
to the function's inputs, preserved from computation to consumption:

```
type TypeBinding {
  name: String
  resolved: Node
  provenance: SubValueRelation   // NEW
  source_param: String           // NEW
}
```

SubValueRelation already exists in std/induction.dag with the right
vocabulary (StrictSubValue, IteratedSubValue, ArithmeticDescent,
PreservedValue, SubValueUnknown). Reuse it — don't reinvent.

**Estimated impact:** ~1365 lines of reconstruction code dissolve in
CX alone. Ownership name-matching dissolves. Emission heuristics
reduce. Total estimated dissolution: ~2000+ lines across all stages.

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
| Tests | `cargo test -p v2-compiler-tests` | GREEN (316 pass) |
| Full DSL | `full_dsl_compiles -- --ignored` | GREEN |
| Diagnostic ratchet | `strict_compile_diagnostic_count -- --ignored` | 424 (honest, non-blocking) |
| L1 gate | `scripts/l1-ratchet.sh --check` | GREEN (0, hard gate) |
| Stage0 freshness | `scripts/check-stage0-freshness.sh` | GREEN (blocking) |

---

## Active work: close the model

All active tracks are now understood as facets of one problem: the IR
doesn't carry enough structure. Each track closes a specific gap.

### Track 1: Provenance on bindings (Lane A + C)

**The highest-leverage fix.** Extend TypeBinding with SubValueRelation
provenance. Dissolves CX reconstruction (33 heuristics, 424
violations) and ownership name-matching.

| Step | What | Status |
|------|------|--------|
| 1 | Add provenance field to TypeBinding, default SubValueUnknown | Not started |
| 2 | Instrument binding sites (params, let, match, lambda, for-each) | Not started |
| 3 | Direct SubValueRelation → LoweringTarget (bypass CallPattern) | Not started |
| 4 | Switch CX to read provenance instead of reconstructing | Not started |
| 5 | Delete reconstruction code (~1365 lines) | Blocked by step 4 |

Design: [docs/cx-design.md §Option B implementation plan](docs/cx-design.md)

### Track 2: Language spec modeling (Lane B)

**Thesis:** Every target language has a spec. Model specs as .dag data
in `dsl/extdeps/languages/`. The emitter reads specs — never decides.

Closes the emission heuristic gap: codegen decisions become
spec-referenced data lookups instead of inline logic.

| Item | Status |
|------|--------|
| LS-1: Type cast rules | Partial (numeric casts validated in infer) |
| LS-2: Operator semantics | DONE (PR #355) |
| LS-3: Expression syntax | Not started |
| LS-4: Ownership/borrowing (Rust) | Partial (needs_sharing parameterized) |
| LS-5: Visibility/module system | Not started |
| LS-6: Shared typed handlers | DONE (PR #355) |

### Track 3: Structural identity / Node.name deletion (Lane A)

**Root cause:** `Node.name` (a string) is used as semantic authority.
Deletion requires declaration-driven identity.

| Item | Status |
|------|--------|
| L1 ratchet (type constructor comparisons) | 0 (hard gate, PR #352) |
| Declaration-driven algebra (Tiers 1-2.5) | DONE |
| source_text_at threading (D6 PR #356, #362) | Mostly done (~20 n.name reads remain) |
| Node.name field deletion | Blocked by remaining n.name reads |

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

### Track 7: Core table dissolution (Lane A + D)

Hand-maintained string-keyed tables in `00_core.dag` should derive
from type declarations in `std/`:

| Table | What it maps | Fix |
|-------|-------------|-----|
| `expr_child_roles` | ExprData variants → accessor functions | Derive from type definition |
| `node_field_roles` | Node fields → structural roles | Derive from type definition |
| `function_size_effects` | Function names → size effects | Function signature metadata |

These dissolve as types move to `std/` and carry their own metadata.

---

## Bootstrap

**Status: COMPLETE.** All stage0 content is 100% generated. Zero
hand-maintained files. Regenerated binary produces identical output
when it self-compiles (fixed-point convergence).

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
