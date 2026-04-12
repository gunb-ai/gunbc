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
  Track 12 (verification)          🟡 ── depends on Track 5 (need working emission)

LAYER 6: Full vision (depends on Layer 5)
  Track 14 (omni-emission)         🔴 ── depends on Track 13; needs vision
  Free consequences (parallelism)  🔴 ── blocked on Tier 1 + ownership + purity
```

| Track | Thesis tier | Readiness | Blocked on |
|-------|------------|-----------|-----------|
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
| Track 13 (single emitter) | Emission is mechanical | 🟡 | Track 2 + 7 |
| Track 11 (runtime safety) | Tier 2 | 🟡 | Design phase |
| Track 12 (verification) | Tier 3 | 🟡 | Track 5 |
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
M1: CX gate → 0 violations (currently 421, ratchet 421)
    Done when: strict_compile_diagnostic_count = 0, gate is blocking
    Key blocker: OUTPUT PROVENANCE on function signatures.
      Same SubValueRelation already on input bindings (S1-S6),
      mirrored to outputs. Not a new system — completes the
      existing pattern. 3 touch points: infer from body, store
      on signature, consumers read at call sites.
    Done: infrastructure (#398), seed data (e61d199),
      classify_argument reads provenance before hardcoded fallback,
      compose_sub_value_relations in std/induction.dag (single
      authority for cross-call composition, conservative on
      IteratedSubValue — identity only). Body inference active
      for non-recursive functions (classify_body_provenance).
    Limitation: output_provenance is List but consumers only
      read |> first (scalar). Per-field consumption not wired.
      Param identity still string-keyed (needs Track 3 ident:Int).
      Body walker is a bootstrap parallel authority — should
      derive from InferScope/TypeBinding.provenance pipeline
      in topo/SCC order once that path exists.
    Next: per-field provenance consumption for product returns
      (Step 3). Wire child-indexed lookup in classify_let_value.
    Unlocks: Stream D (-132), body-inferred categories (-196),
      arithmetic refinement (-44), C3-C6 deletion, tokenizer (-22)
    Remaining ~10 (graph DFS) needs language primitive
    Note: Stream D parser restructuring is DONE mechanically but
      CX can't see through helper returns without output provenance.
      Shelved until provenance lands.

M2: Node.name deleted
    Done when: Node.name field removed, l1-ratchet = 0
    How: fix authored_name_at fallback, eliminate ~15 remaining reads
    Active: quick-owl-889
    Unblocks: Stream B Layer 1 (last-use clone elision)

M3: review.dag runs end-to-end
    Done when: review.dag compiles, builds, runs live against real APIs
    How: fix remaining RE-3 serde gaps
    Mostly done (RE-1/2/4 complete)

M4: Single emitter reads data, never decides
    Done when: 05_emit_rust/python/go.dag deleted, all emission from specs
    How: Lane C (coercion = emission, language plugins)
    Blocked on: M1 + M2 substantially complete
    Design: docs/single-emitter-design.md
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
Target: eliminate 132 CX violations (Category B) by construction.
Design: [src/v2/parser-design.md](src/v2/parser-design.md).
**Current state:** Mechanical restructuring DONE (0 ParserState
references, fixed-point verified, 392/393 tests pass). BUT CX
violations did not decrease — the CX analyzer can't see that helper
return values (e.g. `expect(tokens).tokens`) are sub-lists of the
input. **Blocked on output provenance** (SubValueRelation on function
signatures). Shelved until that lands.
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
| RE-3: review.dag passes live integration | Partial — PR #397 merged (CLI borrow, text responses, shell channels, standalone Cargo.toml). Remaining: emitter heuristics should become structural facts (see below). |
| RE-4: Anthropic REST API end-to-end | Test added (requires ANTHROPIC_API_KEY) |

**RE-3 deferred architectural items** (PR #397 review feedback, M4/M8 direction):
- Shell channel contracts: `emit_shell_return` infers field semantics from
  type shape and field name (Bool→exit code, "stderr"→stderr, String→stdout).
  Upstream fix: extend `ShellTransportConfig` in `dsl/extdeps/transports/shell.dag`
  with explicit channel→field mappings. No types exist yet for this.
- WireFormat structural type: `emit_plain_response_body` infers text vs JSON
  from output type shape. Upstream fix: add `WireFormat`/`ContentType` type to
  `std.serialization` or a new module; attach to response blocks so the emitter
  translates rather than guesses. REST transport has `content_type: String?`
  but it is not leveraged by the emitter.
- Qualified identity for `read_only_params_index`: currently keyed by bare
  `item.name` strings. Should use module-qualified or stable interned identity
  (Track 3 dependency).
- Structural Cargo model: `emit_cargo_toml` uses raw string concatenation.
  `dsl/extdeps/cargo.dag` defines `CargoPackage` but the emitter doesn't use it.

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
- ~~`keyword_to_name` in 02_parse.dag duplicates the tokenizer keyword
  table. Reconcile to single authority.~~
  Resolved: `dag_non_name_keywords` data set in dag/syntax.dag is the
  single authority; `is_name_keyword` uses lookup, no hardcoded strings.
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

**Blocked on:** Track 5 (need working emission to execute against).

### Track 13: Single emitter (compiler-laws.md Lane C)

**Thesis claim:** emission is mechanical translation.

**Current state:** Phase 1 complete, Phase 2 (expression dispatch) complete.
The shared emitter (`05_emit.dag`) has zero language-decision `match target`
branches. Python and Go expression rendering is unified into a single
`emit_unified_typed_expr` dispatcher that reads LanguageSpec data — ~50
per-language functions deleted, CX ratchet 421→416. Per-language emitter
files retain: pattern rendering, TCO, func body, type defs, service defs.
Rust emitter is untouched (ownership logic, Phase 6).

**Progress:** Phase 1 ✓, Phase 2 ✓, Phase 3-4 ready, Phase 5-6 blocked on LS-4.

**Target:** one emitter that reads `LanguageSpec` + `InhabitantDecl`
data per target. Adding a new target language means adding a new
`dsl/extdeps/languages/<lang>/` directory, not touching the compiler.

**Next:** Phase 3 (TCO/block unification), Phase 4 (service/transport).
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
