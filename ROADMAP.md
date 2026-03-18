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
              ┌────────────────────┤
              │                    │
   ┌──────────▼──────────┐  ┌─────▼───────────────────────┐
   │  Result<T,E> in DSL │  │  A1: Gist Compilation (<60s) │
   │  (Blocker 2)        │  │  ← before A6, not before A1  │
   │  ← before A6        │  └──────────────┬──────────────┘
   └─────────────────────┘                 │
                              ┌────────────▼────────────┐
                              │  R8: Rc-wrap generated   │
                              │  types (DAG values =     │
                              │  shared ownership)       │
                              │  ← NEXT BLOCKER          │
                              └─────────────────────────┘

Parallel with R8 (no dependencies):

   ┌──────────────────────┐  ┌──────────────────┐  ┌──────────────────┐
   │ Blocker 3: delete    │  │ C3/C4: Language  │  │ D2-D4: Cost      │
   │ TypeExpr from        │  │ emission from    │  │ analysis on real │
   │ 00_core.dag (~300    │  │ extdeps + CLI    │  │ code             │
   │ lines, mechanical)   │  │                  │  │                  │
   └──────────────────────┘  └──────────────────┘  └──────────────────┘

Deferred (decided, waiting on prerequisites):

   ┌──────────────────────┐  ┌──────────────────┐
   │ Blocker 1: shared    │  │ B3: Expr→Node +  │
   │ emitter walk (A →    │  │ delete validate_ │
   │ needs v2 self-host)  │  │ no_unresolved()  │
   └──────────────────────┘  └──────────────────┘
```

**Critical path:** R8 → A1 → A4 → A5 → A6 → A7

---

## Immediate Priority: R8 — Rc-Wrap Generated Types

**Structural principle:** The DAG language has value semantics with no mutation.
Every value is logically shared — using a value twice doesn't require two
copies. The codegen maps DAG values to Rust ownership, where using a value
twice requires `.clone()`. That's the mismatch. The fix is uniform: **DAG
values are shared; Rust represents sharing as `Rc`.**

```text
DAG value semantics          Rust representation
─────────────────           ────────────────────
String                  →    &str param (Copy)         ← R3 done
Int, Bool               →    i64, bool (Copy)          ← already free
List<T>                 →    Rc<Vec<T>>                ← already done
Map<K,V>                →    Rc<HashMap<K,V>>          ← runtime ops done
struct Node { ... }     →    Rc<Node>                  ← THIS IS R8
enum TokenKind { ... }  →    Rc<TokenKind>             ← THIS IS R8
```

**Rule:** Every non-Copy generated type (struct or non-unit-variant enum) is
Rc-wrapped at all usage sites. `compile_ident` can emit `.clone()` freely on
any variable — it's always O(1). No type-specific checks, no special cases.

**Types excluded from Rc-wrapping** (pure tag enums, already Copy):
Connective, BinOpKind, UnaryOpKind, OperationModifier, RenderTarget, Severity,
Certainty. The codegen already detects these (`is_simple_enum` → derives Copy).

### Callsite migration

**type_codegen.rs** (the structural change — one predicate, applied uniformly):

- **TC-1:** `type_expr_to_rust_with_registry` Named type resolution (~line 150).
  Thread a `HashSet<String>` of non-Copy generated type names. Emit `Rc<T>`
  instead of `T` for matching names. This single change controls field types
  in struct/enum definitions AND function signatures.
- **TC-2/TC-4/TC-5:** `typedef_to_code_ir` and `format_variant` field rendering.
  Inherits from TC-1 — field types become `Rc<T>` automatically.
- **TC-3:** `typedef_to_code_ir_boxed` Box-wrapping. Skip Box for Rc-wrapped
  fields — Rc already heap-allocates, breaking cycles. Extends the existing
  `TypeExpr::Generic` exclusion to Named types in the Rc set. This eliminates
  all current boxed fields (return_type, body, type_annotation, transport,
  config) since their types are all Rc-wrapped.
- **TC-6/TC-7:** `fndef_to_code_ir` param and return type rendering. Inherits
  from TC-1.

**fn_codegen.rs** (construction, matching, field access):

- **FN-1:** `compile_struct_field_value` (~line 1020). When target field is
  `Rc<T>` and value is freshly constructed `T`, wrap in `Rc::new(...)`.
  Mirrors existing Box-wrapping logic.
- **FN-2:** `compile_match_typed` (~line 3427). When scrutinee type is
  Rc-wrapped, emit `match &*x { ... }` instead of `match x { ... }`.
  Bindings become references; field access through Deref still works.
- **FN-3/FN-4:** Box deref logic (`collect_boxed_deref_stmts`, field access
  deref). Remove for fields that are Rc-wrapped (Deref handles automatically).
  Keep for any remaining Box-wrapped fields.

**render_rust.rs:**

- **RR-2:** `render_match`. If deref is inserted at IR level (FN-2), no
  change needed. Otherwise, insert `&*` on Rc-typed scrutinees.

**v2_runtime_shim.rs:**

- **RT-1:** `index_by` closure receives `&Rc<V>` instead of `&V`. Auto-deref
  handles field access in closures — verify but likely no change needed.

**v2_crate_emit.rs:**

- **V2-2/V2-3:** Hardcoded types (`SourceSpan`, `BindingPower`, `FilePath`)
  are NOT Rc-wrapped — they're small, Copy-compatible, not generated from .dag.

### Key simplification: Box-wrapping becomes unnecessary

Once all generated types are Rc-wrapped, `compute_recursive_fields` returns
empty — Rc already breaks all cycles. The R2 boxing of Node.transport/config
becomes redundant. The entire Box-wrapping infrastructure can be simplified
or removed after R8 lands.

### Acceptance

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

## Design Decisions (Decided)

### Blocker 1: Emitter walk triplication → Decision: A (shared walk with callbacks)

Deferred until v2 self-hosted. Requires function-as-data, available in v2 but
not v1 bootstrap. Until then, the bounded duplication (3 backends) is accepted.

### Blocker 2: Fabrication-on-error → Decision: A (add Result<T, E> to DSL)

The structural fix. Large scope but it's a language feature, not a hack. Does
not block A1 critical path — current fabrication pattern works for bootstrap.
Should land before A6 (fixed point) to make the self-hosted compiler
structurally sound.

### Blocker 3: Delete TypeExpr from 00_core.dag → Scheduled for next tasks

Mechanical work: trace last callers of `field_to_node`/`variant_to_node` in
daglang-emit, migrate them, delete ~300 lines. Parallelizable with R8.

### Blocker 4: Node conflates resolved/unresolved → Deferred to B3

`validate_no_unresolved()` violates Invariant 9 (correctness by construction,
not by validation). The validation pass is marked for deletion when B3
(Expr→Node) reworks pipeline boundary types — that's the natural moment to
make resolved-vs-unresolved a type distinction rather than a runtime check.

**TODO in B3:** delete `validate_no_unresolved()` and replace with structural
type distinction (e.g., `ResolvedNode` wrapper or equivalent).

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

Also: delete `validate_no_unresolved()` (Blocker 4). When pipeline boundary
types are reworked here, make resolved-vs-unresolved a type distinction
rather than a runtime check. The validation pass violates Invariant 9.

**Acceptance:**
- [ ] `Expr` type deleted from `00_core.dag`
- [ ] `Typed*` family deleted
- [ ] `validate_no_unresolved()` deleted; replaced with structural type boundary
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
