# gunbc Roadmap

**Goal:** Self-hosted v2 compiler. The compiler is written in `.dag`, compiles
itself, and produces identical output when compiling itself again (fixed point).

**Thesis:** Explicit cause-and-effect relationships with basic primitives
(truth-valued structure, `Conj`/`Disj`, composition) are sufficient to express
any information concept. Named types are aliases for compositions; the compiler
should always be able to see through the name to the structure underneath.

---

## Current State (2026-03-18)

### What works

- **455 tests pass** — 363 daglang-emit + 92 v2-compiler-tests (9 ignored)
- **Generated crate compiles** — `v2_crate_cargo_check` passes in <5s
- **Self-parse + self-resolve** — compiled v2 compiler tokenizes, parses, and
  resolves all 9 .dag modules with zero errors
- **Gist resolve doesn't OOM** — was 10GB SIGKILL; now ~2GB stable
- **String operations are O(1)** — char_at, string_length, substring, scan_*
- **String params don't clone** — &str in generated code, Copy semantics
- **Node shrunk from ~544b to ~120b** — transport/config boxed
- **Interpreter list_push is O(1)** — Arc COW via try_unwrap
- **TCO loops don't leak** — state moved, not cloned

### What's blocked

**Track A (self-hosting) blocked by non-String clone traffic.** The generated
v2 crate takes >20 minutes in release mode for 1,515 lines of gist sources.
R3 eliminated String cloning (~4GB), R6 eliminated O(n²) tokenization, but
struct/enum types (Node, Token, Param, etc.) are still cloned on every
multi-use variable reference. This is the remaining constant-factor multiplier.

### Baseline tests

```bash
cargo test -p daglang-emit --quiet               # 363 tests
cargo test -p v2-compiler-tests --quiet           # 92 tests (9 ignored)
cargo clippy --all-targets -- -D warnings         # clean
cargo test -p v2-compiler-tests v2_crate_cargo_check  # generated crate compiles
```

---

## Dependency Chart

```text
                    ┌─────────────────────────────────────┐
                    │         Self-Hosted Compiler         │
                    │     (A6: fixed point, A7: retire)    │
                    └──────────────┬──────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────┐
                    │       A4-A5: Full Self-Compile       │
                    │   v2 compiles itself → stage 0 → 1   │
                    └──────────────┬──────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────┐
                    │     A1: Gist Compilation (<60s)      │
                    │   gist.dag → tokenize/parse/resolve  │
                    │    /infer/emit in release mode        │
                    └──────────────┬──────────────────────┘
                                   │
              ┌────────────────────┼────────────────────┐
              │                    │                     │
   ┌──────────▼──────────┐  ┌─────▼───────────┐  ┌─────▼───────────┐
   │  R8: Struct clone    │  │ R1-R7: DONE ✓   │  │ S4: Test base   │
   │  reduction           │  │ String/Node/TCO  │  │ DONE ✓ (92+363) │
   │  ← NEXT BLOCKER      │  │ /intrinsics/COW  │  │                 │
   └──────────────────────┘  └─────────────────┘  └─────────────────┘

Independent tracks (can proceed in parallel):

   ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
   │ B3/B4: Node conv │  │ C3/C4: Language  │  │ D2-D4: Cost      │
   │ (needs design    │  │ emission from    │  │ analysis on real │
   │  decisions)      │  │ extdeps + CLI    │  │ code             │
   └──────────────────┘  └──────────────────┘  └──────────────────┘
```

**Critical path:** R8 → A1 → A4 → A5 → A6 → A7

---

## Immediate Priority: R8 — Struct Clone Reduction

The 2026-03-18 session eliminated string cloning (R3) and O(n²) intrinsics
(R6), reducing gist memory from 10GB to ~2GB. But the generated crate still
takes >20 minutes because `compile_ident` clones every non-Copy struct/enum
variable on multi-use. For 1,515 lines of gist source:

- Each Token clone: ~100 bytes × ~25K tokens × ~3 uses = ~7.5MB
- Each Node clone: ~120 bytes × ~5K nodes × ~5 uses = ~3MB
- Each ParseResult clone: ~200+ bytes × thousands of results

These compound inside hot loops (tokenizer, parser, resolver).

**Root cause:** `compile_ident` (fn_codegen.rs) only distinguishes String
params (now &str/Copy) from everything else. All other multi-use variables
get `.clone()`. The codegen has no concept of borrowing for struct types.

**Design options:**

- (A) **Rc-wrap generated struct types.** Generated structs use `Rc<T>` so
  clone is O(1). Requires changing type codegen, not function codegen. The
  generated code already uses Rc for List/Map fields.
- (B) **Reference-based codegen for non-mutated struct params.** Same approach
  as R3 but for all non-Copy types: change function params from `T` to `&T`.
  Larger scope — every struct param becomes a reference, requiring lifetime
  annotations or ownership conversion at use sites.
- (C) **Accept current performance; focus on self-hosting.** The v2 compiler's
  own source (~14K lines) is ~10x larger than the gist (1,515 lines). If gist
  takes 20min, self-compile would take hours. This option is not viable for A4+.

**Recommendation:** Option A (Rc-wrap) is the smallest change with the biggest
impact. It aligns with how List/Map are already wrapped and doesn't require
lifetime tracking.

**Acceptance:**

- [ ] `v2_crate_gist_resolve` passes in release mode in <60 seconds
- [ ] memory usage for gist resolve drops below 500MB
- [ ] all 455 tests pass
- [ ] generated crate compiles clean

---

## Completed Work

### Track R: Representation + Runtime (ALL DONE)

| Item | What | Impact |
|------|------|--------|
| R1 | Type size assertions in generated tests | Node ≤ 160b, TypedExpr ≤ 800b enforced |
| R2 | Box Node.transport + Node.config | Node: ~544b → ~120b |
| R3 | String params → &str in generated code | Eliminates ~4GB string clone traffic |
| R4 | Interpreter list_push Arc COW | O(1) amortized append (30 files) |
| R5 | TCO clone leak fix | Loop state moved, not cloned |
| R6 | O(1) ASCII string intrinsics | char_at, string_length, substring, scan_* |
| R7 | Kernel primitive complexity contracts | dsl/std/primitives.dag |

### Track S: Stabilization (SUBSTANTIALLY DONE)

- **S1 (partial):** Parser builds Nodes directly; TypeExpr functions deleted
  from 04_infer.dag. Remaining: TypeExpr definition + helpers in 00_core.dag
  (see Blocker 3).
- **S4:** 92 v2-compiler-tests pass, v2_crate_cargo_check passes. Generated
  crate compiles. Gist resolve no longer OOMs.
- **S2/S3:** Emit hot paths typed, list builders use O(1) push, Kahn improved.

### Other completed items

- **B2:** `04_typecheck.dag` renamed to `04_infer.dag`
- **C1:** `LanguageSpec` interface in `dsl/std/languages.dag`
- **C2:** Rust, Python, Go language extdeps in `dsl/extdeps/languages/`
- **D1:** Cost algebra types in `dsl/std/complexity.dag`
- **E0:** Monolith artifact wrapper defined
- **P0:** Stack overflow mitigated via stacker at re-entrant call sites
- **Streams 2+3:** PortContract dissolved, Shape → Connective, dead code removed
- **Node convergence (partial):** Field/Param/ResourceUse/TypedExpr/FuncSig/
  TypedNode type fields are Node; all three emitters have node-based readers

---

## Design Decisions Pending

Four structural themes require design decisions before the corresponding work
can proceed. These are independent of the A1 critical path.

### Blocker 1: Emitter walk triplication

Node-type emission, transport dispatch, and service def emission are each
implemented three times (Rust, Python, Go). Adding a container type or
transport binding requires editing all three backends.

**Options:**
- (A) Shared walk parameterized by callbacks (needs function-as-data → v2)
- (B) `TypeShape` classification: shared `classify_node_type()`, backends render
- (C) Accept bounded duplication (3 backends, unlikely to grow)

**Recommendation:** B. Aligns with Invariant 6.

### Blocker 2: Fabrication-on-error pattern

The DSL has no `Result<T, E>`. Parse and inference error paths fabricate dummy
values alongside diagnostics. Parser dummies are tolerable; inference dummies
propagate fabricated types mid-pipeline.

**Options:**
- (A) Add `Result<T, E>` to the DSL. Structural fix, large scope.
- (B) Convention: `ok: Bool` field, pipeline halts on first false.
- (C) Accept parser pattern; fix inference individually.

**Recommendation:** C now, A later.

### Blocker 3: Delete TypeExpr from 00_core.dag

`TypeExpr` definition (8 variants), `type_expr_to_node` + helpers, and
`Predicate` type remain. Parser no longer calls `type_expr_to_node`, but
`field_to_node`/`variant_to_node` are still referenced in the v1 emit pipeline.

**Blocked on:** Tracing last callers in daglang-emit, migrating them, deleting
~300 lines.

### Blocker 4: Node conflates resolved/unresolved

`validate_no_unresolved()` exists because the type boundary is too permissive.

**Options:**
- (A) `ResolvedNode` wrapper. Large scope.
- (B) `resolved: Bool` field. Convention-based.
- (C) Accept runtime validation.

**Recommendation:** C. The validation pass catches violations at typecheck time.

---

## Track A: Self-Hosting

**Dependencies:** R8 (struct clone reduction) → A1 → A4 → A5 → A6 → A7

### A1: Gist compilation

Feed `gist.dag` and transitive dependencies through the v2 pipeline.

**Acceptance:**
- [ ] `v2_crate_gist_resolve` passes in release mode in <60s
- [ ] `v2_compile_gist_rust`: v2 compiles gist → Rust → `cargo check`

### A2: Runtime bridge

Generate entry point and runtime dependencies so compiled gist executes.

**Acceptance:**
- [ ] generated `main.rs` + `Cargo.toml` with runtime deps
- [ ] `cargo run -- gist --dry-run` produces correct output

### A3: Gist end-to-end

**Acceptance:**
- [ ] compiled Rust gist creates a real GitHub gist (manual gate)

### A4: Full self-compile pipeline

Extend self-compile from tokenize/parse/resolve to full pipeline including
infer and emit.

**Acceptance:**
- [ ] v2 crate processes its own .dag source through the full pipeline
- [ ] emitted Rust files compile with `cargo check`
- [ ] no OOM or stack overflow on any .dag file up to 4000 lines
- [ ] self-compile ratchet asserts semantic properties stronger than
      "non-empty file emitted"

### A5: Bootstrap stage 0 → 1

```text
v1 compiles v2 .dag → Rust → rustc → v2-stage0
v2-stage0 compiles v2 .dag → Rust → rustc → v2-stage1
```

### A6: Fixed point

`stage1 output == stage2 output`

### A7: v1 retirement

v2 builds and tests without v1 in the dependency chain.

---

## Track B: Node Convergence

### B1: TypeExpr → Node (MOSTLY DONE)

All typed fields are Node. Remaining: TypeExpr definition + helpers in
00_core.dag (Blocker 3), parser boundary spray, bridge predicate lossy
conversion.

### B2: Rename typecheck → infer (DONE)

### B3: Expr → Node (NEEDS DESIGN DECISION)

Dissolve `Expr` and `Typed*` family into node patterns. After this, "typed"
just means "return_type is filled in."

**Acceptance:**
- [ ] `Expr` type deleted from `00_core.dag`
- [ ] `Typed*` family deleted
- [ ] pipeline shape is `Node → Node → Node → TextFile`

### B4: Transport dissolution (NEEDS DESIGN DECISION)

`TransportBinding` should dissolve. Transport behavior should come from
structure rather than a fixed enum.

---

## Track C: Language Emission

### C1-C2: DONE

### C3: Emitters consult extdeps

Emitters import from language extdeps instead of inline data. Adding a new
target means writing an extdep, not editing compiler logic.

### C4: CLI target selection

`--target` flag loads the appropriate language extdep.

---

## Track D: Runtime Complexity Analysis

### D1: Cost algebra (DONE — types defined in dsl/std/complexity.dag)

### D2: Typed summaries

Infer symbolic summaries from typed expressions/functions. Per-function
`ComplexitySummary` with `work`, `span`, `output_size` as symbolic `CostExpr`.

### D3: DAG composition

Compose summaries over lowered DAG. DAG work = sum of node work, span =
longest dependency path, loop work = iteration count × body work.

### D4: Proofs and reporting

Surface complexity as proof/report. Policy checks can reject unbounded
workflows.

---

## Track E: Artifact Planning

### E0: Monolith wrapper (DONE)

### E1-E4: Artifact model, target placement, boundary semantics, reporting

These define how one `.dag` graph is partitioned into multiple artifacts,
placed onto multiple targets, and emitted with explicit contracts between
pieces. Depends on B4/C3 being far enough along that target facts and
boundary structure are explicit.

---

## Backlog

- Anonymous record target resolution — ambiguous cases must fail closed
- Collection intrinsic semantics in shared IR
- Generated self-hosting tests and stage contracts
- TCO backend contract — no silent partial fallback
- Embedded source metadata through lowering/emission

---

## The Fully Converged Node

After Tracks B and C complete:

```dag
type Connective = Conj | Disj

type Node {
  name: String
  span: SourceSpan
  children: List<Node>
  connective: Connective?
  params: List<Node>
  return_type: Node?
  body: Node?
  transport: Node?
  properties: List<Node>
}
```

### Why each field is irreducible

| Field | Logical role | Why separate |
|-------|--------------|--------------|
| `children` + `connective` | Composition | The core primitive |
| `params` | Obligations | Consumed, not composed |
| `return_type` | Guarantee | Flows out, not in |
| `body` | Proof / computation | How, not what |
| `transport` | I/O grounding | Must remain structural |
| `properties` | Extensible metadata | Domain facts |

### Pipeline

```text
source -> parse -> resolve -> infer -> emit
           |        |         |        |
         Nodes    Nodes     Nodes    TextFiles
          raw     linked    typed
```

One type flows through the pipeline; each phase enriches it rather than
translating into a parallel representation.

---

## The End State

- self-hosted
- structurally unified
- compositional
- target-polymorphic
- artifact-aware
- bootstrap-free
- fixed-point reproducible
