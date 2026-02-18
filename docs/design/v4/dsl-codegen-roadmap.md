# DSL Codegen Roadmap: From .dag to Working Binaries

**Status**: Working Draft — February 2026
**Companion**: [`dsl-design.md`](./dsl-design.md), [`dsl-roadmap.md`](./dsl-roadmap.md)

**Definition of Done**: `daglang compile foo.dag --target {rust,go,c,mips}` produces
compilable/assemblable artifacts that run identically to the hand-built binary.
All targets share a single language DAG — each target is an exit point, not a
separate backend.

---

## Current State

```
Parse → Resolve → Typecheck → Lower → Derive → Emit
  ✓        ✓          ✓          ✓        ✓       STUB
```

The emit phase (`daglang-emit`) is scaffold-only. But significant infrastructure
already exists for the language DAG model:

| Component | Location | Status |
|-----------|----------|--------|
| `code_ir.rs` | `core/ir/src/code_ir.rs` | Target-agnostic AST (Stmt, Expr, Item, FnDef, EnumDef, StructDef) — slightly Rust-flavored |
| `render_ir.rs` | `core/ir/src/render_ir.rs` | OutputMedium trait + CodeRenderer/MarkupRenderer/FrameRenderer traits |
| `language/` | `core/ir/src/language/` | SubDag-composed language model: TuringComplete/ConfigFormat categories, TypeSystemMapping/NamingConventions/CommentPrefix traits, 8+ languages |
| CodeRenderer impls | `core/codegen/src/testgen/` | Rust, Python, TypeScript renderers (stub-level) |
| the-gunbai IR | `the-gunbai/crates/gunbai-ir/` | EdgeKind (DataFlow/Control/TriggerGate), Effect 2-bit, Scatter/RepeatUntil, Understanding-driven codegen |

---

## Architecture: Languages as a DAG

Languages are not independent backends. They are nodes in an abstraction DAG where
each level's IR is the interface the level below must satisfy.

```
                        Computation
                     (from daglang-lower:
                      what each node DOES)
                            │
                            ▼
                       AbstractIR
              (functions, variables, conditionals,
               calls, loops — no language features)
                            │
              ┌─────────────┴─────────────┐
              │                           │
          Turing                       Markup
              │                           │
    ┌─────────┼─────────┐         ┌──────┴──────┐
    │         │         │         │             │
 Systems   Managed   Scripted   Block        Inline
 (Rust)    (Go)     (Python)  (Markdown)    (HTML)
    │         │         │
    └────┬────┘         │
         │              │
     C-style            │
   (C, structs,         │
    pointers,           │
    malloc)             │
         │              │
         ▼              │
     Register           │
   (MIPS, x86:          │
    registers,          │
    syscalls,           │
    labeled blocks)     │
```

### Lowering between levels

Each level defines the same semantic operations with increasing detail:

| Operation | AbstractIR | C-style | Register (MIPS) |
|-----------|-----------|---------|-----------------|
| Store value | `Let(name, expr)` | `Type name = expr;` | `sw $t0, offset($sp)` |
| Call function | `Call(fn, args)` | `fn(a0, a1)` | `move $a0,..; jal fn` |
| Conditional | `If(cond, then, else)` | `if (cond) {..} else {..}` | `beq $t0,$zero,L_else` |
| Iterate | `For(x, items, body)` | `for(int i=0;i<n;i++){..}` | `L_top: bge $t0,$t1,L_end` |
| Return | `Return(expr)` | `return expr;` | `move $v0,$t0; jr $ra` |
| String concat | `BinOp(a, "+", b)` | `snprintf(buf,..,a,b)` | byte-copy loop |
| File write | `Transport(Write,path,data)` | `fwrite(data,1,n,fp)` | `li $v0,4; syscall` |

Emitting at any level means: lower `Computation` → `AbstractIR` → then lower through
the DAG until you reach your target level, then render to text.

### What already exists vs what's needed

**Exists (reuse):**
- `code_ir.rs` Stmt/Expr/Item hierarchy → this IS the AbstractIR (needs minor refactoring)
- `CodeRenderer<M>` trait → this is the "render to text" step at each level
- `TypeSystemMapping` → maps abstract types to concrete per-language
- `NamingConventions` → converts identifiers per-language
- `LanguageOp` SubDag composition → the DAG structure for language relationships

**Needs work:**
- Factor `code_ir.rs` to separate target-agnostic core from Rust-specific extensions
- Add `CStyleIR` lowering (AbstractIR → C constructs)
- Add `RegisterIR` lowering (CStyleIR → register-level instructions)
- Implement `CodeRenderer` for C and MIPS
- Bridge `Computation` (from daglang-lower) to `AbstractIR` (code_ir)

---

## Track A: Computation → AbstractIR Bridge

### A1. Define Computation model
**Files**: new `core/daglang/daglang-emit/src/computation.rs`

The target-independent description of what each DAG node does.
Not "how" (that's language-specific) but "what" (semantically).

```rust
enum Computation {
    /// Pure: read inputs, apply transform, produce outputs
    Pure { inputs: Vec<TypedPort>, outputs: Vec<TypedPort>, body: PureBody },
    /// Transport boundary: world I/O
    Transport { prepare: RequestSpec, execute: TransportKind, parse: ResponseSpec },
    /// Resource acquisition: produce a handle
    ResourceAcquire { handle_type: String, handle_value: String },
    /// Collection: apply operation to list elements
    Collection { kind: CollectionOpKind, element_type: String },
}

enum PureBody {
    Literal(serde_json::Value),       // Hardcoded value (LoadRegistry, FsEnv)
    Template { pattern: String, vars: Vec<String> },  // String interpolation
    StringOp(StringOpKind),           // Concat, Split, Filter, Map operations
    JsonOp(JsonOpKind),               // Extract, Parse, Serialize
    Compare { left: String, right: String },  // Content freshness
    Conditional { condition: String, then_port: String, else_port: Option<String> },
    Aggregate { inputs: Vec<String>, strategy: AggregateKind },  // Multi-input combine
    ServiceCall(ServiceCallMetadata), // Delegated to service handler
}

enum TransportKind {
    FileRead,
    FileWrite,
    FileExists,
    ShellExec,
    HttpRequest,
}
```

Tasks:
- [ ] A1.1 — Define `Computation` enum, `PureBody`, `TransportKind`
- [ ] A1.2 — Define `TypedPort` (name + abstract type + cardinality)
- [ ] A1.3 — Implement `classify_computation(node: &Node<LoweredOp>) -> Computation`
- [ ] A1.4 — Tests: every makegen node → expected Computation variant
- [ ] A1.5 — Tests: every pragma node → expected Computation variant

### A2. Build EmitPlan from DAG
**Files**: new `core/daglang/daglang-emit/src/plan.rs`

Topo-ordered list of computations with data flow.
Every backend consumes this same plan.

```rust
struct EmitPlan {
    steps: Vec<EmitStep>,
    entrypoints: Vec<EntrypointPort>,
    transport_nodes: Vec<String>,
}

struct EmitStep {
    node_id: String,
    computation: Computation,
    input_sources: Vec<InputBinding>,
    output_bindings: Vec<OutputBinding>,
}

enum InputBinding {
    FromStep { step_index: usize, port: String },
    FromEntrypoint { port: String },
    Constant(serde_json::Value),
}
```

Tasks:
- [ ] A2.1 — Define `EmitPlan`, `EmitStep`, `InputBinding`, `OutputBinding`
- [ ] A2.2 — Implement `build_emit_plan(dag, artifacts) -> EmitPlan`
- [ ] A2.3 — Tests: makegen → 10-step plan in topo order
- [ ] A2.4 — Tests: pragma → plan with 3 parallel chains correctly ordered

### A3. Computation → AbstractIR lowering
**Files**: new `core/daglang/daglang-emit/src/lower_to_ir.rs`

Convert EmitPlan steps into `code_ir` constructs (Stmt, Expr, Item).
This is the bridge from "what" to "how" — but still target-agnostic.

```rust
/// Lower an EmitPlan into a SourceFile (code_ir)
fn lower_plan_to_abstract_ir(plan: &EmitPlan) -> SourceFile {
    // Each EmitStep becomes a sequence of Stmts in main()
    // InputBindings become variable references
    // OutputBindings become let-bindings
    // Transport nodes become function calls to a transport API
    // Entrypoints become function parameters
}
```

For makegen, this produces:
```
fn main(path: String) {
    let registry = literal_json({...});              // Step 1: LoadRegistry
    let makefile_content = render_template(...);      // Step 2: RenderMakefile
    let read_request = file_read_request(path);       // Step 3: PrepareRead
    let read_response = execute_transport(read_req);  // Step 4: ExecuteRead
    let fresh = compare(read_response, makefile);     // Step 5: Compare
    if !fresh {                                       // Step 6-8: Conditional write
        let write_request = file_write_request(...);
        execute_transport(write_request);
    }
    report(fresh);                                    // Step 9: Report
}
```

Tasks:
- [ ] A3.1 — Lower `PureBody::Literal` → `Expr::Value` or `Expr::Call` to constructor
- [ ] A3.2 — Lower `PureBody::Template` → `Expr::FormatStr` or string concat chain
- [ ] A3.3 — Lower `PureBody::Compare` → `Expr::BinOp("==", left, right)`
- [ ] A3.4 — Lower `PureBody::Conditional` → `Stmt::If`
- [ ] A3.5 — Lower `Transport` → `Stmt::Let` + `Expr::Call` to transport API
- [ ] A3.6 — Lower `InputBinding::FromStep` → `Expr::Var(step_output_name)`
- [ ] A3.7 — Lower `EntrypointPort` → function parameter
- [ ] A3.8 — Assemble into `SourceFile` with `fn main()`
- [ ] A3.9 — Tests: makegen EmitPlan → expected SourceFile structure
- [ ] A3.10 — Tests: pragma EmitPlan → expected SourceFile with parallel chains

---

## Track B: Factor code_ir into Abstraction Tiers

### B1. Extract AbstractIR core from code_ir
**Files**: `core/ir/src/code_ir.rs`

The existing `code_ir.rs` is 90% target-agnostic but has Rust-specific constructs
mixed in (`Deref`, `RefMut`, `MacroCall`, `ImplBlock`). Factor into tiers.

**Tier 0 — AbstractIR** (truly universal):
```
Stmt: Let, Expr, If, For, Return, Comment, Blank
Expr: Var, Str, IntLit, BoolLit, Call, BinOp, UnaryOp, Field, Array, Block
Item: FnDef, StructDef, EnumDef (without derives)
```

**Tier 1 — SystemsIR** (Rust/C++ extensions):
```
Expr: Deref, Ref, RefMut, MacroCall, Path, Closure
Item: ImplBlock, EnumDef with derives
Stmt: TailExpr (implicit return)
```

**Tier 2 — ManagedIR** (Go/Python extensions):
```
Expr: GoRoutine, Channel, Defer, Slice
Stmt: GoDefer, GoSelect
```

**Tier 3 — CStyleIR** (C lowering target):
```
Expr: Malloc, Free, Cast, SizeOf, AddressOf
Stmt: Goto, Label
Item: Typedef, FunctionPointer
```

**Tier 4 — RegisterIR** (MIPS/x86 lowering target):
```
Instruction: Load, Store, Add, Sub, Mul, Branch, Jump, Syscall
Operand: Register, Immediate, Label, StackOffset
```

Tasks:
- [ ] B1.1 — Audit code_ir.rs: tag each Stmt/Expr variant with its tier
- [ ] B1.2 — Extract Tier 0 (AbstractIR) as base types — all existing code still works
- [ ] B1.3 — Gate Tier 1 (SystemsIR) extensions behind feature or module boundary
- [ ] B1.4 — Define Tier 3 (CStyleIR) types — new, doesn't exist yet
- [ ] B1.5 — Define Tier 4 (RegisterIR) types — new, doesn't exist yet
- [ ] B1.6 — Define lowering trait: `trait LowerIR<From, To> { fn lower(from: &From) -> To; }`

### B2. AbstractIR → SystemsIR lowering (Rust target)
**Files**: new `core/daglang/daglang-emit/src/lower_systems.rs`

Add Rust-specific constructs: ownership, Result, derive macros, use statements.

```rust
fn lower_to_rust(abstract_ir: &SourceFile) -> SourceFile {
    // FnDef → add Result<_, ExecError> return type
    // StructDef → add #[derive(Debug, Clone)]
    // Call to transport API → Result<_, _> with ? operator
    // String values → .to_string() calls
    // Add use/import statements
}
```

Tasks:
- [ ] B2.1 — Add Result wrapping for fallible functions
- [ ] B2.2 — Add derive macros to generated structs/enums
- [ ] B2.3 — Add `use` statements from import analysis
- [ ] B2.4 — String literal → `String` ownership conversion
- [ ] B2.5 — Transport API calls → gunbc-exec or standalone runtime calls
- [ ] B2.6 — Tests: abstract makegen IR → expected Rust-specific IR

### B3. AbstractIR → ManagedIR lowering (Go target)
**Files**: new `core/daglang/daglang-emit/src/lower_managed.rs`

Add Go-specific constructs: multiple returns, error handling, package imports.

```rust
fn lower_to_go(abstract_ir: &SourceFile) -> GoSourceFile {
    // FnDef → add (result, error) multi-return
    // If(err != nil) error handling pattern
    // Package declaration + imports
    // String handling (Go strings are already value types)
}
```

Tasks:
- [ ] B3.1 — Define `GoSourceFile` (or extend code_ir with Go variants)
- [ ] B3.2 — Multi-return error handling pattern
- [ ] B3.3 — Package + import emission
- [ ] B3.4 — Go type mapping via existing `TypeSystemMapping`
- [ ] B3.5 — Tests: abstract makegen IR → expected Go-specific IR

### B4. AbstractIR → CStyleIR lowering
**Files**: new `core/daglang/daglang-emit/src/lower_c.rs`

Lower to C: explicit memory, function pointers, no generics.

```rust
fn lower_to_c(abstract_ir: &SourceFile) -> CSourceFile {
    // StructDef → C struct with tagged union for variant types
    // FnDef → C function with return value + error out-param
    // String → char* with explicit length tracking
    // List → array pointer + count
    // Let → stack allocation or malloc based on escape analysis
    // Cleanup → free() chains or arena scope
}
```

Tasks:
- [ ] B4.1 — Define `CSourceFile` and C-specific AST nodes
- [ ] B4.2 — Value type → C tagged union mapping
- [ ] B4.3 — String handling (char*, length, null-termination)
- [ ] B4.4 — Memory strategy: arena allocator for most allocations
- [ ] B4.5 — Error handling: return code + errno or out-param
- [ ] B4.6 — Tests: abstract makegen IR → expected C IR

### B5. CStyleIR → RegisterIR lowering
**Files**: new `core/daglang/daglang-emit/src/lower_register.rs`

Lower C to MIPS instructions.

```rust
fn lower_to_mips(c_ir: &CSourceFile) -> MipsProgram {
    // Function → label + stack frame setup + jr $ra
    // Local variables → stack offsets
    // Function calls → $a0-$a3 args + jal + $v0 return
    // If → beq/bne + labels
    // String literal → .data section label
    // File I/O → syscall sequences
    // String ops → byte-copy loops
}
```

Tasks:
- [ ] B5.1 — Define `MipsProgram`, `MipsFunction`, `MipsInstruction`
- [ ] B5.2 — Register allocation: linear scan ($t0-$t9 temps, $s0-$s7 saved)
- [ ] B5.3 — Stack frame layout (arguments, locals, saved registers)
- [ ] B5.4 — Calling convention ($a0-$a3 in, $v0 out, $ra save/restore)
- [ ] B5.5 — String operations as byte-copy/compare loops
- [ ] B5.6 — Syscall emission (open/read/write/close/exit)
- [ ] B5.7 — Tests: C makegen IR → expected MIPS program

---

## Track C: Renderers (IR → Text at Each Level)

### C1. Extend existing CodeRenderer for Rust
**Files**: `core/codegen/src/testgen/` (existing renderer stubs)

The `CodeRenderer<M>` trait already exists. The Rust renderer is partially implemented
for testgen. Extend it to handle full SourceFile emission for generated binaries.

Tasks:
- [ ] C1.1 — Render `FnDef` with full signature, body, attributes
- [ ] C1.2 — Render `EnumDef` with derives and variants
- [ ] C1.3 — Render `StructDef` with field types and visibility
- [ ] C1.4 — Render `ImplBlock` with trait and methods
- [ ] C1.5 — Render `Import` as `use` statements
- [ ] C1.6 — Emit `Cargo.toml` with dependencies
- [ ] C1.7 — Tests: Rust IR → expected source text (snapshot tests)

### C2. Go renderer (CodeRenderer impl)
**Files**: new `core/codegen/src/go_renderer.rs` or similar

Tasks:
- [ ] C2.1 — Render Go function definitions with multi-return
- [ ] C2.2 — Render Go struct types
- [ ] C2.3 — Render Go error handling idiom (`if err != nil`)
- [ ] C2.4 — Render Go imports and package declaration
- [ ] C2.5 — Emit `go.mod`
- [ ] C2.6 — Use existing `TypeSystemMapping` for Go type names
- [ ] C2.7 — Use existing `NamingConventions` for Go identifier style (camelCase)
- [ ] C2.8 — Tests: Go IR → expected source text

### C3. C renderer (CodeRenderer impl)
**Files**: new `core/codegen/src/c_renderer.rs`

Tasks:
- [ ] C3.1 — Render C function definitions with prototypes
- [ ] C3.2 — Render C structs with tagged union Value type
- [ ] C3.3 — Render C include directives
- [ ] C3.4 — Render C main() with argc/argv
- [ ] C3.5 — Emit Makefile for C compilation
- [ ] C3.6 — Tests: C IR → expected source text

### C4. MIPS renderer (new trait or RegisterRenderer)
**Files**: new `core/codegen/src/mips_renderer.rs`

Tasks:
- [ ] C4.1 — Render .data section (string literals, constants)
- [ ] C4.2 — Render .text section (functions as labeled blocks)
- [ ] C4.3 — Render instructions (load, store, arithmetic, branch, jump, syscall)
- [ ] C4.4 — Render stack frame prologues/epilogues
- [ ] C4.5 — Tests: MIPS program → expected assembly text

---

## Track D: Exec-Runtime Fast Path (Rust Layer 1)

While Tracks A-C build the language DAG properly, this track gets a working
generated binary ASAP using the existing gunbc-exec runtime.

### D1. Rust exec-runtime codegen
**Files**: `core/daglang/daglang-emit/src/rust_exec_runtime.rs`

Generate Rust code that builds `Dag<Op>` + calls `gunbc-exec`. This bypasses
the language DAG temporarily — it's the bootstrap path.

Tasks:
- [ ] D1.1 — Emit Op enum with one variant per DAG node
- [ ] D1.2 — Emit `impl Executable for Op` with match dispatch
- [ ] D1.3 — Emit executor bodies (port from exec-bridge implementations)
- [ ] D1.4 — Emit graph construction (`Dag::new() + add_node + add_edge`)
- [ ] D1.5 — Emit `fn main()` with CLI arg parsing + execute_and_display
- [ ] D1.6 — Emit `Cargo.toml` with gunbc-ir/gunbc-exec path deps
- [ ] D1.7 — Makegen end-to-end: generated binary produces identical Makefile
- [ ] D1.8 — Pragma end-to-end: generated binary produces identical pragma files

### D2. Reconcile with language DAG
Once Tracks A-C mature, the exec-runtime path becomes one rendering of
`SystemsIR` — the Rust-specific lowering that happens to use gunbc-exec.
Eventually the native Rust path (Track B2) replaces it.

Tasks:
- [ ] D2.1 — Express exec-runtime codegen as AbstractIR → SystemsIR → Rust text
- [ ] D2.2 — Verify output identical to D1 path
- [ ] D2.3 — Remove D1 standalone path (replaced by language DAG path)

---

## Track E: CLI + Testing + CI

### E1. `daglang compile` command
**Files**: `core/daglang/daglang-cli/src/main.rs`

```
daglang compile <input.dag> --target {rust|go|c|mips} [--out <dir>] [--layer 1|2]
```

Tasks:
- [ ] E1.1 — Add `compile` subcommand to CLI parser
- [ ] E1.2 — Wire through driver: parse → lower → derive → plan → lower_ir → render
- [ ] E1.3 — `--target` selects exit point in language DAG
- [ ] E1.4 — `--layer 1` (exec-runtime, Rust only) vs `--layer 2` (native, all targets)
- [ ] E1.5 — Write generated files to `--out` directory

### E2. Cross-language parity test harness
**Files**: new `core/daglang/daglang-cli/tests/codegen_parity.rs`

Tasks:
- [ ] E2.1 — Build and run generated binaries per target (cargo, go build, gcc, mips-as+qemu)
- [ ] E2.2 — Capture output (file content written)
- [ ] E2.3 — Assert identical output across all targets
- [ ] E2.4 — Makegen parity: Rust == Go == C == MIPS Makefile output
- [ ] E2.5 — CI integration: run parity tests on every push

### E3. Obligation-driven test generation
**Files**: new `core/daglang/daglang-emit/src/test_gen.rs`

Tasks:
- [ ] E3.1 — Emit dry-run completion test per target language
- [ ] E3.2 — Emit per-transport-node mock test
- [ ] E3.3 — Emit pure-node snapshot test from NodeIoExample
- [ ] E3.4 — Rust: `#[test]` functions
- [ ] E3.5 — Go: `func Test*` functions
- [ ] E3.6 — C: test runner with assert macros

---

## Track F: Reconciliation with the-gunbai

### F1. Shared abstractions
Align the two repos' models where they overlap.

Tasks:
- [ ] F1.1 — Reconcile EdgeKind: adopt the-gunbai's DataFlow/Control/TriggerGate in gunbc
- [ ] F1.2 — Reconcile Effect model: adopt 2-bit (writes_world × deterministic) classification
- [ ] F1.3 — Reconcile Value types: bridge gunbai Value (Artifact, Secret, Capability) ↔ gunbc Value
- [ ] F1.4 — Reconcile PortType: align gunbai's simpler type system with gunbc's TypeId strings
- [ ] F1.5 — Document shared abstractions in a cross-repo design doc

### F2. Understanding-driven codegen alignment
the-gunbai generates code from Understandings (versioned system knowledge).
The language DAG should be the shared rendering layer.

Tasks:
- [ ] F2.1 — Map gunbai's `CodegenEngine` output to `code_ir::SourceFile`
- [ ] F2.2 — Route gunbai's Rust/Python/TypeScript generation through CodeRenderer<M>
- [ ] F2.3 — Share `TypeSystemMapping` and `NamingConventions` across repos

---

## Execution Order & Dependencies

```
Week 1-2: Foundation
  ├── A1 (Computation model) ──→ A2 (EmitPlan)
  ├── B1 (Factor code_ir into tiers)
  └── D1 (exec-runtime fast path — working binary ASAP)

Week 2-3: Bridge + First Targets
  ├── A3 (Computation → AbstractIR lowering)
  ├── B2 (AbstractIR → SystemsIR/Rust)
  ├── C1 (Rust renderer extension)
  └── D1.7-D1.8 (makegen + pragma e2e)

Week 3-4: Parallel Language Targets
  ├── B3 + C2 (Go: ManagedIR + renderer)
  ├── B4 + C3 (C: CStyleIR + renderer)
  └── E1 (daglang compile CLI)

Week 4-5: Assembly + Parity
  ├── B5 + C4 (MIPS: RegisterIR + renderer)
  ├── E2 (cross-language parity tests)
  └── D2 (reconcile exec-runtime with language DAG)

Week 5-6: Polish
  ├── E3 (test generation)
  ├── F1-F2 (the-gunbai reconciliation)
  └── More workflows (codegen, build, bootstrap)
```

### Critical path

```
A1 (computation) → A2 (plan) → A3 (lower to IR) → B2 (Rust) + B3 (Go) + B4 (C)
                                                          │
B1 (factor code_ir) ─────────────────────────────────────┘
                                                          │
D1 (exec-runtime) ── parallel, delivers working binary ──→ D2 (reconcile)

B4 (C-style) → B5 (Register/MIPS) → C4 (MIPS renderer) → E2 (parity)
```

### What can run in parallel

- **D1** (exec-runtime) is independent — start day 1, delivers working binary fast
- **A1-A3** (computation model) is independent of D1
- **B2, B3, B4** (language tier lowerings) can all proceed in parallel once B1 + A3 land
- **B5** (MIPS) depends on B4 (C) — C is the stepping stone
- **C1-C4** (renderers) each parallel with their corresponding B-track lowering
- **F1-F2** (reconciliation) can start anytime, no blocking dependency

---

## Gap Analysis

| Component | Exists | Needed | Effort |
|-----------|--------|--------|--------|
| **Track A: Computation → IR** | | | |
| Computation model | No | Classify all ~95 ops | M |
| EmitPlan (topo steps + data flow) | No | Build from DAG + derive | M |
| Computation → AbstractIR lowering | No | Map each PureBody to code_ir | L |
| **Track B: IR Tier Factoring** | | | |
| AbstractIR (Tier 0) | 90% (code_ir.rs) | Factor out Rust-specific bits | S |
| SystemsIR (Tier 1, Rust) | Exists mixed in code_ir | Separate as extension | S |
| ManagedIR (Tier 2, Go) | No | Define Go-specific constructs | M |
| CStyleIR (Tier 3, C) | No | Define C constructs | M |
| RegisterIR (Tier 4, MIPS) | No | Define instruction model | L |
| AbstractIR → Rust lowering | No | Ownership, Result, derives | M |
| AbstractIR → Go lowering | No | Multi-return, error idiom | M |
| AbstractIR → C lowering | No | malloc, structs, pointers | L |
| C → MIPS lowering | No | Register alloc, syscalls | XL |
| **Track C: Renderers** | | | |
| Rust CodeRenderer | Partial (testgen stubs) | Full SourceFile rendering | M |
| Go CodeRenderer | No | Full Go file rendering | M |
| C CodeRenderer | No | Full C file rendering | M |
| MIPS RegisterRenderer | No | Assembly text rendering | M |
| **Track D: Exec-Runtime** | | | |
| Rust exec-runtime codegen | No | Op enum + Executable + main | L |
| **Track E: CLI + Testing** | | | |
| `daglang compile` CLI | No | Subcommand + driver wiring | S |
| Cross-language parity | No | Build + run + compare framework | M |
| Test generation | No | From TestObligations | M |
| **Track F: Reconciliation** | | | |
| EdgeKind alignment | Separate models | Adopt DataFlow/Control/TriggerGate | M |
| Effect model alignment | Different taxonomies | Adopt 2-bit model | S |
| Shared rendering layer | Separate codegen engines | Route through CodeRenderer | L |

**S** < 1 day, **M** 1-3 days, **L** 3-5 days, **XL** 5-8 days

---

## Success Criteria

1. `daglang compile dsl/tools/makegen.dag --target rust` → cargo build → correct Makefile
2. `daglang compile dsl/tools/makegen.dag --target go` → go build → identical Makefile
3. `daglang compile dsl/tools/makegen.dag --target c` → gcc → identical Makefile
4. `daglang compile dsl/tools/makegen.dag --target mips` → mips-as + qemu → identical Makefile
5. All four targets share the same `Computation → AbstractIR → LevelIR → Text` pipeline
6. Adding a new language = implementing a level's lowering + renderer (not a new backend)
7. Pragma compiles to at least Rust and Go with output parity
8. Cross-language parity test runs in CI

---

## Relationship to Existing Documents

| Document | Scope | Overlap |
|----------|-------|---------|
| `dsl-roadmap.md` Part 1 | DSL Build | **Complete** — this starts where Part 1 ends |
| `dsl-roadmap.md` Part 2 | Migration | This is the **prerequisite** — codegen before migration |
| `dsl-design.md` §Emit | 13 emission targets | This implements targets via language DAG (not per-target backends) |
| `TODO_URGENT_dsl_migration.md` | Migration checklist | "Ready Now" items become Track D1.7+ |
| `code_ir.rs` | Target-agnostic AST | Becomes AbstractIR (Tier 0) with factoring |
| `render_ir.rs` | OutputMedium + renderers | CodeRenderer<M> is the rendering interface at each tier |
| `language/mod.rs` | Language SubDag compositions | Provides TypeSystemMapping + NamingConventions for all targets |
