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
DAG vocabulary reconciliation: [docs/dag-vocabulary-reconciliation.md](docs/dag-vocabulary-reconciliation.md)

---

## Four themes — dependency order

The roadmap is organized around 4 thesis-aligned themes in strict
dependency order. Each theme has a clear done-criterion and a
structural `.dag` model that defines its interfaces.

```
THEME 1: Close the Binding Model
  Binding unification (7 → 2) ──→ Triple provenance → single
    ──→ Output provenance on sigs ──→ CX gate → 0
  Proposed in: docs/binding-model-proposal.md
  Done when: CX violations = 0, gate blocking, classify_* deleted
  Absorbs: Track 1, Stream A, Stream C, Stream D, M1

THEME 2: Structural Identity
  authored_name_at fix ──→ n.name reads → 0 ──→ Node.name deleted
  Done when: Node.name field removed, l1-ratchet = 0
  Absorbs: Track 3, M2, Track 6 remainder

THEME 3: Ownership as Dimension
  Depends on: Theme 2 (stable binding identity)
  Ownership lattice in std/ ──→ Ownership on bindings
    ──→ Layer 1 (last-use) ──→ Layer 2 (post-TCO)
    ──→ LS-4 design ──→ Layer 3 (borrow propagation)
  Proposed in: docs/binding-model-proposal.md
  Done when: analyze_ownership deleted, ownership on TypeBinding
  Absorbs: Stream B, LS-4, Track 2 (partial)

THEME 4: Emission as Data
  Depends on: Themes 1-3 (provenance + identity + ownership)
  Single emitter Phase 5 ──→ Phase 6 (Rust unification)
  ValueContext implementation ──→ Core table dissolution
  Done when: one emitter reads LanguageSpec per target
  Absorbs: Track 13, Track 4, Track 7, Track 2 (partial)
```

| Theme | Readiness | Blocked on | Key metric |
|-------|-----------|------------|------------|
| **1: Close the Binding Model** | Design/Implement | SVR composition rules + ownership table open questions | CX violations: 340 → 0 |
| **2: Structural Identity** | Implement | authored_name_at fallback fix | n.name reads: ~15 → 0 |
| **3: Ownership as Dimension** | Design | Theme 2 (stable binding identity) | Separate pass → dimension on bindings |
| **4: Emission as Data** | Implement/Design | Theme 1 (Rust needs provenance), LS-4 (borrow model) | Per-language files: 3 → 0 |

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
- **Complexity:** 33 heuristics in annotate_descent_evidence (340 violations)
- **Ownership:** string name matching for fold accumulators
- **Emission:** compensates for upstream information loss (M2)
- **Core tables:** hand-maintained string-keyed semantic maps

### The fix: binding unification + provenance on bindings

**Step 1:** Reduce 7 syntactic binding forms to 2 fundamental mechanisms
(`std/binding.dag`):

The binding form question is not a separate type — it's derivable
from edge position: does the binding sit in `params` (caller provides
SVR) or `body`/`children` (expression determines SVR)? The caller
context (fold/descend/direct call) is the `SubValueRelation` the
call site attaches. The access shape is the SVR derived from the
expression. All reduce to existing SVR vocabulary.

See [docs/binding-unification-design.md](docs/binding-unification-design.md).
Full DAG vocabulary accounting: [docs/dag-vocabulary-reconciliation.md](docs/dag-vocabulary-reconciliation.md).

**Step 2:** Thread the existing `SubValueRelation` (std/induction.dag)
through bindings — preserved from computation to consumption:

```
type TypeBinding {
  name: String
  resolved: Node
  provenance: SubValueRelation   // reuses existing authority
}
```

SubValueRelation already has the right vocabulary (StrictSubValue,
IteratedSubValue, ArithmeticDescent, PreservedValue, SubValueUnknown)
with InductiveField and ShrinkFactor. No new type — single authority.

**Step 3:** Collapse the triple/quadruple classification system to
one consumer:

| Current | Role | After |
|---------|------|-------|
| `classify_binding_provenance` (04_infer.dag ~2616) | At bind time | Single SVR computation at binding creation |
| `classify_let_value` / `classify_argument` (04_infer.dag ~2649-3104) | For descent evidence | Reads TypeBinding.provenance (no re-derivation) |
| `classify_body_provenance` (04_infer.dag ~3851) | For output provenance | Reads TypeBinding.provenance (no re-derivation) |
| `classify_self_call_evidence` (complexity.dag ~2535) | CX fallback | Deleted (reads annotated evidence only) |

Implementation plan: [src/v2/cx-design.md §Option B](src/v2/cx-design.md).

---

## Theme 1: Close the Binding Model

*Absorbs: Track 1 (provenance), Stream A, Stream C, Stream D,
M1 (CX gate → 0), binding unification.*

The critical path. Every downstream consumer (CX, ownership, emission,
future dimensions) benefits from fewer binding forms and richer
provenance.

### Structural modeling (proposed)

Proposed `.dag` types in [docs/binding-model-proposal.md](docs/binding-model-proposal.md):

- **Edge vocabulary** — `SubValueRelation` (already in std/induction.dag)
  IS the edge classifier. Binding form (parameter vs let-binding) is
  derivable from Node field position. Caller context and access shape
  are both SVR values, not separate types.

- **Only new type** — `UsageEdge = Consumed | Read | Projected | Threaded`
  (what happens at each use site, orthogonal to SVR).

- **Dimension table** — SVR-keyed: one row per SVR variant, one column
  per dimension. Replaces the triple classification system.

Types land in `dsl/std/` when implementation work begins.

### Path to CX gate = 0 (340 → 0)

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
  Depends on: Steps 1-2 (reliable per-argument evidence).

Step 4: Universal checker replaces classify_* heuristics ~-80
  is_valid_proof becomes the single termination authority.
  Delete 5+ ad-hoc classification functions, hardcoded
  fallback tables (C3-C6). One function, not five heuristics.
  Depends on: Step 3.

Step 5: Remaining edge cases                            ~-10
  Graph DFS, arithmetic refinement.

Violation landscape (340 total):
  104 parse_type_expr + 89 render_node_type + 48 to_string
  = 241 (57%) from recursive functions with multi-param
  descent patterns (lexicographic, Steps 2-3).
  132 from parser struct returns (per-field, Step 1).
  ~47 from other patterns (arithmetic, graph DFS, Step 4-5).

### Binding unification migration (incremental)

Option B: late desugaring (after inference, before CX/ownership).
Each desugaring is a separate PR:

- PR 1: for-each → fold (most obvious desugaring)
- PR 2: match arm bindings → let + field access
- PR 3: lambda param variants → single boundary-crossing edge with
  SVR provided by call site

Dimension computation uses only SVR on the edge.

### Done when

- CX violations = 0, gate is blocking
- `classify_argument`, `classify_self_call_evidence`,
  `collect_evidence_incremental` deleted or reduced to thin readers
- `lambda_param_provenance` on InferScope dissolved (SVR from call
  site replaces the side-channel)
- Adding new syntax requires parser + desugaring only

### Active workboards

- **CX:** [src/v2/cx-design.md §Workboard](src/v2/cx-design.md)
- **Parser (blocked on output provenance):**
  [src/v2/parser-design.md](src/v2/parser-design.md)

---

## Theme 2: Structural Identity

*Absorbs: Track 3 (Node.name deletion), M2, Track 6 remainder
(algebra field dispatch).*

`Node.name` (a string) is used as semantic authority throughout
the compiler. Deletion requires declaration-driven identity and
enables stable binding identity for Theme 3.

### Current state

| Item | Status |
|------|--------|
| L1 ratchet (type constructor comparisons) | 0 (hard gate, PR #352) |
| Declaration-driven algebra (Tiers 1-2.5) | DONE |
| source_text_at threading (D6 PR #356, #362) | Mostly done (~20 n.name reads remain) |
| Per-file source_index at resolve boundary (PR #378) | DONE |
| Migrate accessor callers to `_at` variants (PR #378) | DONE — 109 sites migrated |
| InternTable as identity consumer | Pending: table exists, threading deferred |
| Fix `authored_name_at` fallback | Blocked: cross-module span mismatch |
| Node.name field deletion | Blocked by authored_name_at + ~15 direct reads |
| Algebra field dispatch (Track 6 remainder) | `find_child_named` still string-keyed |

### Done when

- Node.name field removed from 00_core.dag
- l1-ratchet = 0
- InternTable is the identity authority
- `find_child_named` replaced by structural child lookup

---

## Theme 3: Ownership as Dimension

*Absorbs: Stream B (clone elision), LS-4 (borrow model), Track 2
(language spec, partial).*

Move ownership from a separate name-keyed pass to a dimension on
bindings following DECLARE/COMPUTE/CARRY/ENFORCE (same architecture
as provenance).

### Structural modeling (proposed)

Proposed `.dag` types in [docs/binding-model-proposal.md](docs/binding-model-proposal.md):

- `OwnershipKind = Owned | Borrowed | Shared` as a `BoundedLattice`
  inhabitant with meet/join/top/bottom
- `UsageEdge = Consumed | Read | Projected | Threaded`
- `ownership_at_svr(source, svr)` — binding-site rule keyed on SVR
- `AccumulatorOwnership = ThreadedOwned | ThreadedShared` — fold
  accumulator contract

Types land in `dsl/std/ownership.dag` when implementation work begins.

### Current compiler state (gap analysis)

The compiler's `src/v2/ownership.dag` has:
- `EdgeKind = Consumed | Read | Threaded | Projected` (parallel to
  `std/ownership.dag`'s `UsageEdge` — to be unified)
- `BindingUsage { name, binding_kind, consumers }` — name-keyed
- `OwnershipDecision = SoleOwner | SharedError | Unclassified`
- `analyze_ownership` — separate pass, walks typed body after inference
- Fold detection via string name matching ("init" arg, method name "fold")

Target: `analyze_ownership` deleted. Ownership facts live on
TypeBinding (or DimensionTable). Fold detection reads SVR
(`PreservedValue` for accumulator, `IteratedSubValue` for element)
instead of matching method name strings.

### Three layers to clone elimination

Stage0 emits ~13,724 `.clone()` calls (verified 2026-04-12).

| Layer | Size | Blocked on | Impact (est.) |
|-------|------|-----------|---------------|
| 1. Last-use elision | 1-2 PRs | Theme 2 (stable binding identity) | ~2,000-4,000 clones |
| 2. Post-TCO ownership | 1 PR | Nothing | ~500-1,000 clones |
| 3. Borrow propagation | 3-5 PRs | LS-4 design | ~15,000-18,000 clones |

**Layer 3** is where ~90% of unnecessary clones live. Read-only
function parameters should be borrowed (`&Rc<T>`) instead of owned
(`Rc<T>`). Requires LS-4 language spec work: add borrow syntax to
SharingStrategy per target language.

### LS-4: Borrow model design

1. Add borrow syntax to SharingStrategy — how does each target
   language express "I only need to read this"?
2. For each function, determine which params are read-only.
   Derivable from ownership proof (all edges are Read/Projected).
3. Emit function signatures with borrows.
4. Cascade: changes function signatures across entire emitted
   codebase — every call site must match. Atomic with stage0 regen.

### Done when

- `analyze_ownership` (separate pass) deleted
- Ownership facts live on TypeBinding or DimensionTable
- OwnershipKind computed at binding sites during inference
- Fold detection reads SVR from bindings, not method name strings
- Layer 3 borrow propagation eliminates bulk of clones

---

## Theme 4: Emission as Data

*Absorbs: Track 13 (single emitter), Track 4 (codegen), Track 7
(core table dissolution), Track 2 (language spec, partial).*

"Emission is mechanical translation" — the emitter reads specs,
never decides.

### Single emitter progress

Phase 1-3.5 complete. Phase 4 verified. Phase 5 in progress.
Python and Go per-language files: no language-decision branches for
expressions, patterns, TCO, block statements, or func bodies.
31 per-language functions deleted, replaced by shared + LanguageSpec.

Line counts: Python 666, Go 689, shared 2983, **Rust 5863.**
Rust is untouched (ownership logic). Phase 6 blocked on LS-4.

### Codegen correctness

| Item | Status |
|------|--------|
| CG-1: Authority consolidation | DONE |
| CG-2: Expression-level gaps (TLC-1..4) | TLC-1/2/3 done, TLC-4 partial |
| CG-3: Parameterization (3 backends → 1) | Phases 1-3 done, 4-6 deferred |

### Core table dissolution

Hand-maintained string-keyed tables in `00_core.dag` dissolve as
types move to `std/` and carry structural facts:

| Table | Fix |
|-------|-----|
| `expr_child_roles` | Derive from type definition |
| `node_field_roles` | Derive from type definition |
| `function_size_effects` | Structural contracts in std/ |
| `keyword_to_name` | Reconcile with tokenizer keyword table |

### ValueContext (planned, not yet implemented)

DESIGN.md describes `ValueContext = ConstantData | RuntimeValue |
SpecificationWitness | CallableValue` as precomputed on EmitGraphInfo.
This does not exist in code yet. The emitter currently uses
`shared_types`, `TypeSummary.has_fn_fields`, and ownership maps.
ValueContext is the structural replacement.

### Done when

- One emitter reads LanguageSpec + InhabitantDecl per target
- Adding a new target language = adding a spec directory
- Per-language files (05_emit_rust/python/go.dag) deleted or
  reduced to spec data only
- expr_child_roles / node_field_roles / function_size_effects deleted

---

## Deprioritized: Modeling Quality

These are real improvements that do not block the 4 themes above.
They apply M9 (DFS the concept DAG) to extdeps and std/ types.

### Record shape dedup (former Track 9)

| Pair | Fix |
|------|-----|
| `CargoDependency` / `CrateDep` | Identical fields — merge to one |
| `TransportResponse` / `HttpResponse` | Identical fields — merge |
| `ShellResponse` / `CliResult` | Same 3 fields, different order — merge |
| Dual `Credential` | Different schemas — rename cloud one |

### Lattice inhabitant consolidation (former Track 8)

`FermiDepth` manually reimplements lattice join. `Set<T>` mapped to
BooleanAlgebra in string table but not structurally. Phase 1 DONE
(DescentEvidence as BoundedLattice). Phase 2 blocked on user-defined
generic emission.

### Extdeps modeling fidelity (former Track 10)

Typed enums exist but service I/O uses String/Bool instead:
- Gist visibility: `Bool` → `GistVisibility`
- PR state: `String` → `PullRequestState`
- LLM stop reason: `String` → typed enum
- LLM model: `String` → typed model enum

---

## Deprioritized: Meta-process Modeling (M5)

Validates the thesis claim that `dag run` is the primary workflow.
Important but separate from compiler correctness.

### Completed

- Phase 0 (interpreter): DONE (PR #409)
- Phase 1 (bootstrap modeling): DONE (PR #418)
  `compiler.dag` as single authority, `ci.dag` gates derived,
  `dag run check_l1_ratchet` end-to-end

### Planned (not blocking themes)

- **Phase 2 (tool modeling, former Track 15):** Replace PATH-based
  resolution with explicit tool registry + upsert. `CliTool`,
  `InstallSource`, `ResolvedTool` in `dsl/extdeps/tools.dag`.
- **Phase 3 (CI YAML emission, former Track 16):** ci.yml becomes
  generated artifact from typed Workflow declaration using
  `extdeps/github/actions.dag`. Shape B (.dag program emits YAML).
- **Phase 3.5 (wire unused modeling, former Track 17):** ~770 lines
  of declared types with zero consumers. Make existing modeling
  load-bearing before adding new modeling.
- **Phase 3.6 (error taxonomy, former Track 18):** Unify per-workflow
  Result coproducts via `std/errors.dag` with `ErrorClass` +
  `Retryability`.
- **Phase 4 (bootstrap verification):** Per-stage structural diffs,
  stage contracts, automatic strategy selection.

---

## CI gates

| Gate | Command | Status |
|------|---------|--------|
| Lint | `cargo clippy --workspace -- -D warnings` | GREEN |
| Tests | `cargo test -p v2-compiler-tests` | GREEN (394 pass) |
| Full DSL | `full_dsl_compiles -- --ignored` | GREEN |
| Diagnostic ratchet | `strict_compile_diagnostic_count -- --ignored` | 340 (honest, non-blocking) |
| L1 gate | `scripts/l1-ratchet.sh --check` | GREEN (0, hard gate) |
| Stage0 freshness | `scripts/check-stage0-freshness.sh` | GREEN (blocking) |

---

## Bootstrap

**Status: COMPLETE.** Stage0 content is generated from .dag source.
Regenerated binary produces identical output when it self-compiles
(fixed-point convergence).

Hand-maintained files in stage0 (not generated, survive regen):

| File | Category | Convert to .dag? |
|------|----------|-----------------|
| `compiler_tests.rs` | Test harness | Generated by emitter |
| `v2_interpreter.rs` | Interpreter | Blocked: bootstrap chicken-and-egg |
| `cli_run.rs` | CLI Run handler | Blocked: calls interpreter |

```
.dag source ──(v2-compiler)──▶ stage0 .rs ──(cargo/rustc)──▶ v2-compiler binary
     ▲                                                              │
     └──────────────────────────────────────────────────────────────┘
```

---

## PERF: Eliminate unnecessary work

**Status (2026-04-12):** Profiling identified fact re-derivation
(not clones) as the dominant bottleneck. PR #422 contains the
fix (merge_envs + data-only skip + InternTable wiring), measured
at ~50% self-compile speedup.

**The lesson:** The biggest win was NOT clone elimination. It was a
6-line fix to `merge_envs` that eliminated fact re-derivation.
`.clone()` count is UNCHANGED (13,724). The speedup had nothing
to do with clones.

| Metric | Current | Target |
|--------|---------|--------|
| Self-compile wall time | 37.6s | <30s |
| `.clone()` in stage0 | 13,724 | Not the priority metric |
| `name.clone()` (heap alloc) | ~1,188 | 0 (via Theme 2 ident:Int) |
| node.name reads in compiler | 107 | 0 |

**The 5 rules:**
1. Profile first.
2. Audit for re-derivation, not just clones.
3. When you thread a fact forward, DELETE the old reconstruction.
4. Watch for fact-flow violations in reviews.
5. Log every case as a KF-2 target.

Design: [docs/perf/clone-elimination.md](docs/perf/clone-elimination.md)

---

## Killer features

Capabilities grounded in the closed-model property.

| Feature | Status | Blocked on |
|---------|--------|------------|
| KF-1: Complexity proof on every compile | 340 violations (non-blocking) | Theme 1 |
| KF-2: Reject suboptimal algorithms | Not built | KF-1 |
| KF-3: Verification from structure | L0 done, L4-L7 not built | Theme 4 |
| KF-4: Cross-language equivalence | Partial | Theme 4 |
| KF-5: Decidable high-level language | **Working** | — |
| KF-6: Hardware target (Verilog/SPICE) | Design only | KF-1 |
| KF-7: Space complexity | TCO detection working | Theme 1 |
| KF-8: Optimality gate | Deferred | KF-1 |

---

## Future work (Tier 2-3)

### Runtime safety (Tier 2)

No internal operation can fail at runtime. Zero coverage today.
Design direction: either prove preconditions at compile time
(refinement types) or make all operations total (return Option).

### Verification from structure (Tier 3)

The compiler generates verification from declarations. L4 (semantic
correctness) partially unblocked. L7 (algebraic law verification)
has first candidate: effects derivation generates `f(f(x)) == f(x)`
test obligations. See [src/v2/tests/testing-strategy.md](src/v2/tests/testing-strategy.md).

### Omni-emission

One intent graph, many artifacts. Blocked on Theme 4 (single
emitter). See THESIS.md for the full vision.

---

## Experimental

### Work orchestration (OaaS)

The management plane as a `.dag` workflow on cloud infrastructure.
Automated PR review (review.dag) validates the approach. CI modeling
(ci.dag) prevents forgotten gates. Orchestration is the next level.

---

## Public release gates

The release is the conjunction of all gates. No partial credit.

### Gate 1: Causal engine is closed (Tier 1)

| Criterion | Test | Theme |
|-----------|------|-------|
| Provenance on bindings | 0 CX violations, reconstruction deleted | Theme 1 |
| Complexity gate blocking | CostUnknown = compile error | Theme 1 |
| Language specs modeled | No inline target-language knowledge | Theme 4 |
| Node.name deleted | l1-ratchet = 0, field deleted | Theme 2 |
| Codegen from structural authority | CG acceptance criteria met | Theme 4 |
| Real program compiles and runs | review.dag end-to-end | Theme 4 |
| Performance | No test >2s, self-compile <30s | PERF |

### Gate 2: Runtime safety (Tier 2)

| Criterion | Test |
|-----------|------|
| All runtime operations total | No `.force()`, no unchecked division |
| Checked arithmetic | Overflow = compile error or checked op |
| Bounds safety | Out-of-bounds = compile error or Option return |

### Gate 3: Verification from structure (Tier 3)

| Criterion | Test |
|-----------|------|
| Semantic correctness (L4) | Emitted code executes, matches .dag evaluation |
| Cross-language equivalence (L5) | Same .dag → same behavior in all targets |
| Decidable language | Working (already met) |

### Gate 4: Demo quality

| Criterion | Test |
|-----------|------|
| One impressive demo | Compile .dag service → show Rust + Python → run live → show proofs |
| Documentation | README, getting-started, language reference |
| Clean install | `cargo install gunbc` works |

### Release dependency chain

```
Theme 1 (binding model) ──→ KF-1 (CX gate) ──→ Gate 1
Theme 2 (identity) ──→ Gate 1
Theme 3 (ownership) ──→ Gate 1 (performance)
Theme 4 (emission) ──→ Gate 1
                                                    │
Runtime safety ─────────────────────────────────→ Gate 2
                                                    │
Verification ───────────────────────────────────→ Gate 3
                                                    │
                          Gate 1 + Gate 2 + Gate 3 ──→ Gate 4 ──→ Release
```

---

## Cross-reference: former track → theme mapping

For continuity with prior discussions and PR descriptions:

| Former | Name | Now in |
|--------|------|--------|
| Track 1 | Provenance | Theme 1 |
| Track 2 | Language spec | Theme 3 (LS-4) + Theme 4 |
| Track 3 | Node.name | Theme 2 |
| Track 4 | Codegen | Theme 4 |
| Track 5 | Real program | Theme 4 (done) |
| Track 6 | Algebra dispatch | Theme 2 |
| Track 7 | Core tables | Theme 4 |
| Track 8 | Lattice inhabitants | Deprioritized: Modeling Quality |
| Track 9 | Record dedup | Deprioritized: Modeling Quality |
| Track 10 | Extdeps fidelity | Deprioritized: Modeling Quality |
| Track 11 | Runtime safety | Future (Tier 2) |
| Track 12 | Verification | Future (Tier 3) |
| Track 13 | Single emitter | Theme 4 |
| Track 14 | Omni-emission | Future |
| Track 15 | CLI tool modeling | Deprioritized: Meta-process |
| Track 16 | CI YAML emission | Deprioritized: Meta-process |
| Track 17 | Wire unused modeling | Deprioritized: Meta-process |
| Track 18 | Error taxonomy | Deprioritized: Meta-process |
| Stream A | Provenance pipeline | Theme 1 |
| Stream B | Clone elision | Theme 3 |
| Stream C | std/ foundation | Theme 1 |
| Stream D | Structural parser | Theme 1 (blocked on output provenance) |
| M1 | CX gate → 0 | Theme 1 |
| M2 | Node.name deleted | Theme 2 |
| M3 | review.dag end-to-end | Done (PR #409) |
| M4 | Single emitter | Theme 4 |
| M5 | Meta-process | Deprioritized |
