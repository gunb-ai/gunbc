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
- **Test baseline:** 89 pass, 0 fail, 9 ignored (gated on stages below).

---

## Stage 1: Gist end-to-end

Validate the full pipeline on a real workload before attempting self-hosting.

### 1a: Gist compilation

Feed gist.dag + 11 transitive dependencies through the v2 pipeline. Verify
emitted code compiles in each target language.

**Acceptance:**
- [ ] `v2_compile_gist_rust` — v2 compiles gist → Rust → `cargo check`
- [ ] `v2_compile_gist_python` — v2 compiles gist → Python → `py_compile`

### 1b: Runtime bridge

Generate entry point and runtime dependencies so the compiled gist executes.

**Acceptance:**
- [ ] Generated `main.rs` + `Cargo.toml` with runtime deps
- [ ] `cargo run -- gist --dry-run` produces correct dry-run output
- [ ] Python equivalent produces same dry-run output

### 1c: End-to-end execution

**Acceptance:**
- [ ] Compiled Rust gist creates a real GitHub gist (manual gate, requires token)
- [ ] Compiled Python gist creates a real GitHub gist (manual gate)

---

## Stage 2: Full self-compile pipeline

Extend the existing `self_compile_all_modules` test from stages 1-3
(tokenize → parse → resolve) to stages 1-5 (+ typecheck + emit). S85
SCC fix may have unblocked the OOM that previously prevented this.

**Acceptance:**
- [ ] v2 crate processes its own .dag source through full pipeline
- [ ] Emitted Rust files compile (`cargo check`)
- [ ] No OOM, no stack overflow on any .dag file up to 4000 lines

---

## Stage 3: Bootstrap

### 3a: Stage 0→1

```
v1 compiles v2 .dag → Rust → rustc → v2-stage0  (what we have today)
v2-stage0 compiles v2 .dag → Rust → rustc → v2-stage1  (the new thing)
```

**Acceptance:**
- [ ] v2-stage1 builds successfully
- [ ] v2-stage1 passes the same test suite as v2-stage0

### 3b: Fixed point

```
v2-stage1 compiles v2 .dag → Rust → rustc → v2-stage2
```

**Acceptance:**
- [ ] stage1 output == stage2 output (compiler reproduces itself)

### 3c: v1 retirement

Once the fixed point holds, v1 is bootstrap scaffolding — no longer needed.

**Acceptance:**
- [ ] v2 builds and tests without v1 in the dependency chain
- [ ] S76-S81 heuristics in `v2_crate_emit.rs` are dead code
- [ ] Interpreter (`daglang-eval`) is optional (dev/REPL, not required)

---

## Stage 4: Node convergence

Structural unification — one type (Node) flows through the entire pipeline.
Each step is validated by re-bootstrapping: modify .dag source, self-compile,
verify fixed point holds. 13 design decisions are made (see memory doc
`project_typeexpr_node_convergence.md`).

### 4a: TypeExpr → Node

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

**Acceptance:**
- [ ] TypeExpr type deleted from 00_core.dag
- [ ] Field, Variant types deleted (replaced by child Nodes)
- [ ] Typechecker works on Nodes, not TypeExpr
- [ ] Fixed point holds after migration

### 4b: Rename typecheck → infer

After convergence, the phase completes a Node graph (fills in return_types),
not checks a separate type system.

**Acceptance:**
- [ ] `04_typecheck.dag` → `04_infer.dag`
- [ ] Fixed point holds

### 4c: Expr → Node

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
- [ ] Fixed point holds

### 4d: Transport dissolution

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
- [ ] Fixed point holds

---

## Stage 5: Language emission as extdeps

Languages are external systems with specifications. They belong in extdeps,
modeled the same way GitHub and Git are modeled.

### Architecture

Three layers, separated:

1. **Interface (compiler-owned):** The compiler defines what facts it needs
   from any target language — type mappings, syntax patterns, naming
   conventions, runtime ops, error model, async model, import system.
   Stable contract, defined as .dag types.

2. **Language extdeps (spec-derived):** Each language fills in the interface
   from its real specification. Evolves independently of the compiler.

3. **Wiring (compiler-owned):** Connects `--target` CLI flag to the
   appropriate language extdep. Trivial for now, eventually dynamic.

```
dsl/extdeps/languages/
  rust/       — types, syntax, runtime, naming, imports, errors, async
  python/     — types, syntax, runtime, naming, imports, errors, async
  typescript/ — ...
  go/         — ...
```

### Kernel runtime resolves naturally

"How do you concat strings in Rust?" is a fact about Rust, captured in
`dsl/extdeps/languages/rust/runtime.dag`. The current `v2_rt.rs` runtime
shim dissolves into the Rust language extdep.

### Per-language emitters shrink

The 1000+ line emitter monoliths (`05_emit_rust.dag`, etc.) shrink to thin
semantic renderers handling irreducible differences (Rust ownership, Python
exceptions, Go multi-return). Surface knowledge (spelling, naming, type
mappings) comes from the language extdep.

**Acceptance:**
- [ ] Language extdeps in `dsl/extdeps/languages/` for Rust, Python
- [ ] Compiler defines LanguageSpec interface
- [ ] Emitters consult extdeps — no hardcoded type/naming maps
- [ ] `--target` CLI flag (default: Rust)
- [ ] Adding a new target = writing a language extdep (no compiler changes)
- [ ] Emitted code identical for all existing test cases
- [ ] Fixed point holds

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
