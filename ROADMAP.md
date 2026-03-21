# gunbc Roadmap

**Goal achieved:** Self-hosted v2 compiler with proven fixed point (stage1 == stage2
byte-identical). The compiler is written in `.dag`, compiles itself, and produces
identical output when compiling itself again.

**Thesis:** Explicit cause-and-effect relationships with basic primitives
(truth-valued structure, `Conj`/`Disj`, composition) are sufficient to express
any information concept. Named types are aliases for compositions; the compiler
should always be able to see through the name to the structure underneath.

---

## What's done

Self-hosting pipeline complete. All structural convergence landed:

- **A1-A7:** Full pipeline, self-compile (0 errors), bootstrap, fixed point, v1 retired
- **B1-B4:** TypeExpr→Node, Expr→ExprData on Node, Transport→Node (all sum types dissolved)
- **C1-C4:** Language extdeps, `--target` CLI flag
- **P1b:** EmitGraph normalization (`04a_normalize.dag`)
- **R1-R9:** Codegen ownership, Rc-wrap, TCO clone strip, fold-accum extract

92 v2-compiler-tests pass, 460+ workspace tests, generated crate compiles clean.

```bash
cargo test -p v2-compiler-tests --lib --quiet     # unit tests
cargo test --workspace --quiet                     # full workspace
```

---

## Track D: Runtime Complexity Analysis

**Eliminates performance fallbacks. All prerequisites met (P1b landed).**

### D1: Cost algebra (DONE — types defined in dsl/std/complexity.dag)

### D-ownership: Static ownership proof (eliminate try_unwrap fallbacks)

Track D builds on P1b's edge classification. P1b computes **semantic consumer
count** (after removing administrative edges like TCO threading and pass-through
reads). Track D strengthens this:
- If semantic_consumers == 1 at a try_unwrap site: emit `Rc::into_inner().expect()`
  (no fallback, panic on violation).
- If semantic_consumers > 1: compile error — "cannot guarantee O(1) mutation,
  restructure the code."
- If P1b hasn't classified a binding: that is a completeness bug, not a
  Track D gap.

This replaces ALL `Rc::try_unwrap(x).unwrap_or_else(|rc| (*rc).clone())`
instances (14 codegen + 4 runtime sites) with statically verified moves.

### D2: Typed summaries

Infer symbolic summaries from typed expressions/functions. Per-function
`ComplexitySummary` with `work`, `span`, `output_size` as symbolic `CostExpr`.

**CostExpr growth risk:** Symbolic formulas can grow faster than the
graph they summarize. Mitigation:
- `CostExpr` must be a shared DAG with interning/hash-consing
- Memoize summaries per function or SCC
- **Ratchet:** CostExpr node count per function/module must be bounded

### D3: DAG composition

Compose summaries over lowered DAG. Span = longest dependency path,
loop work = iteration count × body work.

- Exclusive branches: condition cost + max(branches), not sum
- Recursive SCC summaries: infer linear/divide-and-conquer, error on
  unresolvable — never silently produce unbounded

### D4: Proofs and reporting

Surface complexity as proof/report. Policy checks can reject unbounded
workflows.

### Track D acceptance

- [ ] `CostExpr` node count per function bounded by ratchet
- [ ] `CostExpr` uses hash-consing / interning (no duplicated subtrees)
- [ ] Recursive SCC summaries: infer linear/divide-and-conquer, annotate
  others, error on unresolvable
- [ ] Exclusive-branch composition uses `max(branches)`, not `sum`

---

## Track E: Artifact Planning

### E0: Monolith wrapper (DONE)

### E1-E4: Artifact model, target placement, boundary semantics, reporting

These define how one `.dag` graph is partitioned into multiple artifacts,
placed onto multiple targets, and emitted with explicit contracts between
pieces. B4 and C3 prerequisites are met.

---

## Track F: Debuggability

**Motivation:** A `.dag` program compiles to an intermediate language (Rust, Python,
JS) which compiles again to machine code or bytecode. When something goes wrong at
runtime, the user sees a Rust panic or Python traceback — pointing at generated code
they didn't write. The gap between "where the error is reported" and "where the error
was authored" can be two compilation steps wide.

**Design principle:** The interpreter is the primary debugging surface. Users debug
their `.dag` logic in `.dag` terms. Cross-language source mapping is an optimization
for production tracing.

**Core interface — TraceEvent:**

```dag
type TraceEvent
  = Enter { node_id: String, span: SourceSpan, inputs: Map<String, String> }
  | Exit  { node_id: String, span: SourceSpan, output: String }
  | Error { node_id: String, span: SourceSpan, message: String }

type TraceFrame {
  func_name: String
  span: SourceSpan
  bindings: Map<String, String>
}

type Trace {
  events: List<TraceEvent>
  stack: List<TraceFrame>
}
```

### F1: Span preservation + interpreter source locations

- Stop discarding spans in emitters — carry SourceSpan through to output
- Thread source spans through interpreter stack frames
- Errors format as `resolve.dag:142:5: type mismatch`

### F2: Interpreter debugger

- Define TraceEvent + TraceFrame types in `dsl/std/`
- Instrument `eval_body`/`eval_stmt` to emit TraceEvents
- Breakpoints by source location or node name
- Step into/over/out mapped to DAG node entry/exit
- Trace recording for offline replay

### F3: Hermetic reproduction

Because every phase is pure and values are immutable, any function
boundary is a potential isolation point. Capture inputs → self-contained test.

### F4: Cross-language source mapping

Source map emission alongside generated code. Error remapper translates
target-language panics back to `.dag` spans. May not be needed if
interpreter debugger (F2) covers user needs.

---

## Multi-Walk Refactor Program

One O(N) walk is fine. K separate O(N) walks over the same collection is
a scaling dimension. The two anti-patterns to remove:
1. **module-level multi-pass** — same items walked in separate passes
2. **result-unpack** — `map(process)` then 2-3 more walks to split fields

| Priority | Area | Planned response |
|----------|------|------------------|
| P0 | `04_reconcile.dag:typecheck_module` (~42 extra walks) | Fuse around wider `ItemContribution` / `ModuleContext` types. One env build, one contribution fold. |
| P1 | `04_reconcile.dag:resolve_expr_types` (8 unpack walks) | Direct accumulation instead of map+split. |
| P2 | `04_reconcile.dag:infer_expr` (5 unpack sites) | Mechanical `fold_unpack` first, inline hottest sites later. |
| P3 | `04_reconcile.dag:resolve_item_types` (7 unpack sites) | Inline accumulation. |
| P4 | `05_emit*.dag` (registry rebuild + arg rewalks) | Widen `ResolvedGraph` to carry shared metadata. |
| P5 | `03_resolve.dag` (kahn/module rewalks) | Defer unless profiling promotes. |

---

## Backlog

- Anonymous record target resolution — ambiguous cases must fail closed
- Collection intrinsic semantics in shared IR
- Generated self-hosting tests and stage contracts
- TCO backend contract — no silent partial fallback
- B3 Ph2a Contract 2 — SCC-aware return type resolution (not yet blocking)

| Item | What |
|------|------|
| General generic syntax | `type Foo<T> = ...` parameterized types. Special-cased Result/Option sufficient for now. |
| Full linear type checking | Prove ownership flow statically in v2 compiler. Use-count-based proof (D-ownership) is sufficient for now. |
| Widen V5 | Handle non-takeable modified fields in functional record update. Current conservative V5 covers hot paths. |

### Invariant to add

**No flags in codegen.** Boolean flags that change compilation behavior
globally (like `force_clone`) are forbidden. Every compilation decision must
be derived from the actual type and context of the expression being compiled,
not from a global check. Flags silently degrade and are impossible to remove
incrementally.

---

## The Fully Converged Node

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
- debuggable (errors trace to `.dag` source; failures reproduce hermetically)
