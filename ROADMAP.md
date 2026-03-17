# gunbc Roadmap

**Goal:** Self-hosted v2 compiler. The compiler is written in .dag, compiles
itself, and produces identical output when compiling itself again (fixed point).

**Thesis:** Explicit cause-and-effect relationships with basic primitives
(truth-valued structure, Conj/Disj, composition) are sufficient to express
any information concept. Named types are aliases for compositions — the
compiler can always see through a name to the structure underneath.

---

## Completed work

- **Stream 2 (sustainability cleanup):** All stale docs deleted. SUSTAINABILITY.md
  updated. S83 fixed (stacker), S84 closed (TCO verified), S85 terminal fix (SCC),
  S76-S81 marked terminal (die with v1).
- **Stream 3 phases A-E:** PortContract dissolved. Shape dissolved via
  `Connective = Conj | Disj`. Dead code deleted (TypeBody, bridge helpers, old
  emit functions). All dissolution comments cleaned.
- **P0 (S85):** Recursive types — SCC cycle detection on type dependency graph.
- **P1 (S84):** TCO pass — verified working in v2 emitter.
- **P1 (S83):** Stack overflow — stacker wrapping at re-entrant call sites.
- **B2:** Rename `04_typecheck.dag` → `04_infer.dag`. Module declaration updated.
- **C1:** LanguageSpec interface defined in `dsl/std/languages.dag` with 14 facets
  (statements, expressions, control_flow, literals, modules, functions, errors,
  type_defs, patterns, async_model, collection_ops, string_ops, map_ops,
  null_coalesce). Compositions: `rust_spec`, `go_spec`, `python_spec`.
- **C2:** Language extdeps modeled for Rust (5 files), Python (5 files), Go (5 files)
  in `dsl/extdeps/languages/`. Each covers types, runtime, imports, errors, async.
- **B1 prep:** TypeExpr↔Node conversion infrastructure complete in `00_core.dag`
  (`type_expr_to_node`, `node_to_type_expr_full`). Dual-write fields
  (`resolved_node: Node?`) added to TypeBinding/ResolveResult/SpanType.
  Node-reading type emit functions (`emit_rust_node_type`, `emit_py_node_type`,
  `emit_go_node_type`) added to all three backends — ready to switch when
  04_infer.dag populates `resolved_node`.
- **Perf audit:** Five-pass audit of v2 compiler performance documented in
  `src/v2/PERF_AUDIT.md`. Identified five repeatable bottleneck patterns:
  quadratic builders, linear-scan lookups, redundant inference, full-tree cloning,
  and duplicated block-emission. Priority order: 04_infer > 01_tokenize >
  03_resolve > 05_emit* > 02_parse/06_pipeline.
- **Test baseline:** 887 pass, 9 fail (module_graph discovery tests — pre-existing
  parse failures on template strings in extdep .dag files).

---

## Parallel tracks

Work is organized into tracks that can proceed independently. Dependencies
between tracks are noted; within each track, steps are sequential.

```
Track A: Pipeline validation        Track B: Node convergence      Track C: Language emission
(gist → self-compile → bootstrap)   (TypeExpr → Expr → transport)  (extdeps model)
─────────────────────────────────   ──────────────────────────────  ─────────────────────────
A1: Gist compilation                B1: TypeExpr → Node             C1: LanguageSpec ✓
A2: Runtime bridge                  B2: Rename typecheck → infer ✓  C2: Rust/Python/Go ✓
A3: Gist end-to-end                 B3: Expr → Node                 C3: Emitters consult
A4: Full self-compile pipeline      B4: Transport dissolution            extdeps
A5: Bootstrap stage 0→1                                             C4: --target CLI
A6: Fixed point
A7: v1 retirement

BLOCKER: Track A gated on v2 perf fixes (see PERF_AUDIT.md).
         B1 completion gated on 04_infer.dag perf fixes (same file).
```

**Dependencies:**
- A5 (bootstrap) requires A4 (full self-compile)
- A6 (fixed point) requires A5
- B1-B4 are validated by re-bootstrapping (requires A6), but design work
  and implementation can begin before A6 on the current v1-bootstrapped compiler
- C1-C4 are fully independent — can proceed in parallel with A and B
- B4 (transport dissolution) benefits from C2 (transport facts live in extdeps)

---

## Track A: Pipeline validation → bootstrap → self-hosting

**Blocker:** v2 compiler performance. Five bottleneck patterns documented in
`src/v2/PERF_AUDIT.md` make self-compile infeasible (quadratic tokenizer,
repeated inference in 04_infer.dag, O(M^4) resolve, O(B^2) emitters).
Perf fixes must land before A4 can proceed.

### A1: Gist compilation

Feed gist.dag + 11 transitive dependencies through the v2 pipeline. Verify
emitted code compiles in each target language.

**Acceptance:**
- [ ] `v2_compile_gist_rust` — v2 compiles gist → Rust → `cargo check`
- [ ] `v2_compile_gist_python` — v2 compiles gist → Python → `py_compile`

### A2: Runtime bridge

Generate entry point and runtime dependencies so the compiled gist executes.

**Acceptance:**
- [ ] Generated `main.rs` + `Cargo.toml` with runtime deps
- [ ] `cargo run -- gist --dry-run` produces correct dry-run output
- [ ] Python equivalent produces same dry-run output

### A3: Gist end-to-end execution

**Acceptance:**
- [ ] Compiled Rust gist creates a real GitHub gist (manual gate, requires token)
- [ ] Compiled Python gist creates a real GitHub gist (manual gate)

### A4: Full self-compile pipeline

Extend `self_compile_all_modules` from stages 1-3 (tokenize → parse → resolve)
to stages 1-5 (+ typecheck + emit). S85 SCC fix may have unblocked the OOM.

**Acceptance:**
- [ ] v2 crate processes its own .dag source through full pipeline
- [ ] Emitted Rust files compile (`cargo check`)
- [ ] No OOM, no stack overflow on any .dag file up to 4000 lines

### A5: Bootstrap stage 0→1

```
v1 compiles v2 .dag → Rust → rustc → v2-stage0  (what we have today)
v2-stage0 compiles v2 .dag → Rust → rustc → v2-stage1  (the new thing)
```

**Acceptance:**
- [ ] v2-stage1 builds successfully
- [ ] v2-stage1 passes the same test suite as v2-stage0

### A6: Fixed point

```
v2-stage1 compiles v2 .dag → Rust → rustc → v2-stage2
```

**Acceptance:**
- [ ] stage1 output == stage2 output (compiler reproduces itself)

### A7: v1 retirement

Once the fixed point holds, v1 is bootstrap scaffolding — no longer needed.

**Acceptance:**
- [ ] v2 builds and tests without v1 in the dependency chain
- [ ] S76-S81 heuristics in `v2_crate_emit.rs` are dead code
- [ ] Interpreter (`daglang-eval`) is optional (dev/REPL, not required)

---

## Track B: Node convergence

Structural unification — one type (Node) flows through the entire pipeline.
After A6, each step is validated by re-bootstrapping (fixed point holds).
Before A6, implementation proceeds on the v1-bootstrapped compiler and is
validated by the existing test suite (887+ tests).

13 design decisions are documented in `project_typeexpr_node_convergence.md`.

### B1: TypeExpr → Node

Dissolve TypeExpr (8 variants) into Node patterns. The typechecker walks
Nodes via connective + children instead of pattern-matching TypeExpr variants.

Types that dissolve: TypeExpr, Field, Variant, TypeBinding, ResolveResult,
ContainerKind, Predicate, FuncSig, FuncEnv.

Key design decisions:
- Type variables are Params on the parent Node (same mechanism as value params)
- Containers/Optional/Map are Nodes with type-level params, defined in std
- Instantiation (`List<String>`) is composition — filling a param
- Refined types are Conj(base, predicate)
- Primitives dissolve — String, Int, Bool are kernel Nodes

**Current state:** Dual-write infrastructure in place. Conversion functions
(`type_expr_to_node`, `node_to_type_expr_full`) complete in 00_core.dag.
Node-reading type emit functions added to all three backends. Remaining:
04_infer.dag must populate `resolved_node` fields (blocked on perf fixes
to the same file), then emitter call sites switch from `emit_type_expr` to
`emit_*_node_type`, then TypeExpr is deleted.

**Acceptance:**
- [x] Conversion infrastructure (type_expr_to_node / node_to_type_expr_full)
- [x] Dual-write fields on TypeBinding, ResolveResult, SpanType
- [x] Node-reading type emit functions (emit_rust/py/go_node_type)
- [x] is_type_alias_return_node shared helper
- [ ] 04_infer.dag populates resolved_node (gated on perf fixes)
- [ ] Emitter call sites switch to Node readers
- [ ] TypeExpr type deleted from 00_core.dag
- [ ] Field, Variant types deleted (replaced by child Nodes)
- [ ] 887+ tests pass / fixed point holds (whichever gate is available)

### B2: Rename typecheck → infer ✓

After convergence, the phase completes a Node graph (fills in return_types),
not checks a separate type system.

**Acceptance:**
- [x] `04_typecheck.dag` → `04_infer.dag`

### B3: Expr → Node

Dissolve Expr (17 variants) into Node patterns. Expressions become Nodes
whose body/children carry computation structure.

Types that dissolve: Expr, TypedExpr, TypedNode, TypedNamedArg, TypedMatchArm,
TypedFieldInit, TypedStringPart, MatchPattern, FieldBinding, LiteralValue,
BinOpKind, UnaryOpKind, StringPart, NamedArg, FieldInit, MatchArm.

After this, "typed" just means "return_type is filled in." One type: Node.
Inference is `List<Node> → List<Node>` — same Nodes, return_types completed.

**Acceptance:**
- [ ] Expr type deleted from 00_core.dag
- [ ] Typed* family deleted (TypedNode, TypedExpr, etc.)
- [ ] `typed_expr_to_expr` conversion deleted
- [ ] Pipeline is `Node → Node → Node → TextFile`

### B4: Transport dissolution

`transport: Node?` — the field stays (structural awareness: no smuggling I/O),
but TransportBinding (the hardcoded 4-variant enum) dissolves. Transport value
becomes a composed Node whose children carry transport facts.

The emitter derives behavior from structure (has `base_url`? → HTTP client;
has `argv`? → subprocess), not from matching a variant tag. New transports
don't require compiler changes.

Types that dissolve: TransportBinding, ServiceConfig, AuthConfig, HeaderDef, EnvDef.

**Acceptance:**
- [ ] TransportBinding enum deleted
- [ ] Emitters derive transport behavior from Node structure
- [ ] `transport != none` is the only hardcoded transport knowledge

---

## Track C: Language emission as extdeps

Languages are external systems with specifications. They belong in extdeps,
modeled the same way GitHub and Git are modeled. Fully independent of
tracks A and B — can proceed in parallel.

### Architecture

Three layers, separated:

1. **Interface (compiler-owned):** The compiler defines what facts it needs
   from any target language — type mappings, syntax patterns, naming
   conventions, runtime ops, error model, async model, import system.
   Stable contract, defined as .dag types.

2. **Language extdeps (spec-derived):** Each language fills in the interface
   from its real specification. Evolves independently of the compiler.

3. **Wiring (compiler-owned for now):** Connects `--target` CLI flag to the
   appropriate language extdep. Trivial, eventually dynamic.

```
dsl/extdeps/languages/
  rust/       — types, syntax, runtime, naming, imports, errors, async
  python/     — types, syntax, runtime, naming, imports, errors, async
  typescript/ — ...
  go/         — ...
```

### C1: Define LanguageSpec interface ✓

The compiler defines the contract: what facts does the emitter need?

**Acceptance:**
- [x] LanguageSpec type defined in `dsl/std/languages.dag` (14 facets)
- [x] Covers: type mappings, naming, syntax patterns, runtime ops,
      error model, async model, import system
- [x] Full compositions: `rust_spec`, `go_spec`, `python_spec`

### C2: Rust, Python, and Go language extdeps ✓

Model each language from its real specification, implementing the
LanguageSpec interface.

Kernel runtime resolves here: "how do you concat strings in Rust?" is a
fact in `dsl/extdeps/languages/rust/runtime.dag`. The current `v2_rt.rs`
dissolves into the Rust language extdep.

**Acceptance:**
- [x] Language extdeps in `dsl/extdeps/languages/` for Rust, Python, Go
- [x] Runtime ops captured (string, list, map operations per language)
- [x] Each language: types.dag, runtime.dag, imports.dag, errors.dag, async.dag

### C3: Emitters consult extdeps

The 1000+ line emitter monoliths shrink to thin semantic renderers handling
irreducible differences (Rust ownership, Python exceptions). Surface
knowledge comes from the language extdep.

**Current state:** Type maps, container templates, and keywords are centralized
as data declarations in `05_emit.dag` (lines 996-1046). These are the single
source of truth but are still inline data, not imported from language extdeps.
The emitters don't yet import from `dsl/extdeps/languages/`. The v1 bootstrap
cannot pass data-declared Maps as function parameters, so per-language
functions use direct `lookup()` on local data (05_emit.dag lines 1072-1190).

**Acceptance:**
- [x] Type/keyword/container data centralized (05_emit.dag data declarations)
- [ ] Emitters import from language extdeps instead of inline data
- [ ] Adding a new target = writing a language extdep (no compiler changes)
- [ ] Emitted code identical for all existing test cases

### C4: CLI target selection

**Current state:** `RenderTarget = Rust | Python | Go` exists in 00_core.dag.
Pipeline dispatches on target in 06_pipeline.dag. No CLI flag yet.

**Acceptance:**
- [x] RenderTarget enum and pipeline dispatch
- [ ] `--target` CLI flag (default: Rust, supports Python, TypeScript, Go)
- [ ] Target selection loads appropriate language extdep

---

## The fully converged Node

After stages 4 and 5 complete:

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
|-------|-------------|-------------|
| `children` + `connective` | Composition (AND/OR of sub-propositions) | The core primitive |
| `params` | Obligations — what must be supplied (IMPLIES antecedent) | Consumed, not composed |
| `return_type` | Guarantee — what is produced (IMPLIES consequent) | Flows out, not in |
| `body` | Proof — computation connecting params to return_type | HOW, not WHAT |
| `transport` | I/O grounding — where this node touches external reality | Must be structural (no smuggling) |
| `properties` | Extensible metadata | Domain facts |

### The irreducible kernel

Only three things can't be Nodes:
1. **Node** — the universal container (circular if self-defined)
2. **Connective = Conj | Disj** — the logical primitive
3. **Kernel primitives** (String, Int, Bool, List, Map) — engineering atoms

Everything else is composition. Named types are aliases. The `type` keyword
is surface sugar that produces a Node.

### Pipeline

```
source → parse → resolve → infer → emit
           ↓        ↓        ↓       ↓
         Nodes    Nodes    Nodes   TextFiles
         (raw)  (imports  (types
                 linked)  filled)
```

One type flows through the entire pipeline. Each phase enriches the same
Nodes rather than converting between representations.

---

## The end state

- **Self-hosted:** written in .dag, compiled by itself
- **Structurally unified:** one type (Node) through the entire pipeline
- **Compositional:** everything is Conj/Disj + kernel primitives
- **Target-polymorphic:** Rust, Python, TypeScript, Go from same source
- **Bootstrap-free:** no v1 dependency, no interpreter dependency
- **Verified by fixed point:** compiler reproduces itself
- **Extensible without compiler changes:** new transports, new languages,
  new domain models — all .dag compositions

---

## Non-goals

- Deleting keywords from surface syntax. Keywords are good parse sugar.
- Expanding String to bits at compile time. The logical decomposition is
  the model; backends render efficiently.
- A single template-driven renderer. Irreducible semantic differences
  between languages (ownership, exceptions, multi-return) stay as thin
  per-language modules.
