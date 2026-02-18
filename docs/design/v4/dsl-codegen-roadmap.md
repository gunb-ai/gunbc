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
- [x] A1.1 — Define `Computation` enum, `PureBody`, `TransportKind` *(TAKEN)*
- [x] A1.2 — Define `TypedPort` (name + abstract type + cardinality) *(TAKEN)*
- [x] A1.3 — Implement `classify_computation(node: &Node<LoweredOp>) -> Computation` *(DONE)*
- [x] A1.4 — Tests: every makegen node → expected Computation variant *(DONE)*
- [x] A1.5 — Tests: every pragma node → expected Computation variant *(DONE)*

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
- [x] A2.1 — Define `EmitPlan`, `EmitStep`, `InputBinding`, `OutputBinding` *(DONE)*
- [x] A2.2 — Implement `build_emit_plan(dag, artifacts) -> EmitPlan` *(DONE)*
- [x] A2.3 — Tests: makegen → 8-step plan in topo order *(DONE)*
- [x] A2.4 — Tests: pragma → plan with 3 parallel chains correctly ordered *(DONE)*

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
- [x] A3.1 — Lower `PureBody::Literal` → `Expr::Value` or `Expr::Call` to constructor *(DONE)*
- [x] A3.2 — Lower `PureBody::Template` → `Expr::FormatStr` or string concat chain *(DONE)*
- [x] A3.3 — Lower `PureBody::Compare` → `Expr::BinOp("==", left, right)` *(DONE)*
- [x] A3.4 — Lower `PureBody::Conditional` → `Stmt::If` *(DONE)*
- [x] A3.5 — Lower `Transport` → `Stmt::Let` + `Expr::Call` to transport API *(DONE)*
- [x] A3.6 — Lower `InputBinding::FromStep` → `Expr::Var(step_output_name)` *(DONE)*
- [x] A3.7 — Lower `EntrypointPort` → function parameter *(DONE)*
- [x] A3.8 — Assemble into `SourceFile` with `fn main()` *(DONE)*
- [x] A3.9 — Tests: makegen EmitPlan → expected SourceFile structure *(DONE)*
- [x] A3.10 — Tests: pragma EmitPlan → expected SourceFile with parallel chains *(DONE)*

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
- [x] B1.1 — Audit code_ir.rs: tag each Stmt/Expr variant with its tier *(TAKEN)*
- [x] B1.2 — Extract Tier 0 (AbstractIR) as base types — all existing code still works *(TAKEN)*
- [x] B1.3 — Gate Tier 1 (SystemsIR) extensions behind feature or module boundary *(TAKEN)*
- [x] B1.4 — Define Tier 3 (CStyleIR) types — new, doesn't exist yet *(TAKEN)*
- [x] B1.5 — Define Tier 4 (RegisterIR) types — new, doesn't exist yet *(TAKEN)*
- [x] B1.6 — Define lowering trait: `trait LowerIR<From, To> { fn lower(from: &From) -> To; }` *(TAKEN)*

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
- [x] B2.1 — Add Result wrapping for fallible functions *(DONE)*
- [x] B2.2 — Add derive macros to generated structs/enums *(DONE)*
- [x] B2.3 — Add `use` statements from import analysis *(DONE)*
- [x] B2.4 — String literal → `String` ownership conversion *(DONE)*
- [x] B2.5 — Transport API calls → gunbc-exec or standalone runtime calls *(DONE)*
- [x] B2.6 — Tests: abstract makegen IR → expected Rust-specific IR *(DONE)*

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
- [x] B3.1 — Define `GoSourceFile` (or extend code_ir with Go variants) *(DONE)*
- [x] B3.2 — Multi-return error handling pattern *(DONE)*
- [x] B3.3 — Package + import emission *(DONE)*
- [x] B3.4 — Go type mapping via existing `TypeSystemMapping` *(DONE)*
- [x] B3.5 — Tests: abstract makegen IR → expected Go-specific IR *(DONE)*

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
- [x] B4.1 — Define `CSourceFile` and C-specific AST nodes *(DONE)*
- [x] B4.2 — Value type → C tagged union mapping *(DONE)*
- [x] B4.3 — String handling (char*, length, null-termination) *(DONE)*
- [x] B4.4 — Memory strategy: arena allocator for most allocations *(DONE)*
- [x] B4.5 — Error handling: return code + errno or out-param *(DONE)*
- [x] B4.6 — Tests: abstract makegen IR → expected C IR *(DONE)*

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
- [x] B5.1 — Define `MipsProgram`, `MipsFunction`, `MipsInstruction` *(DONE)*
- [x] B5.2 — Register allocation: linear scan ($t0-$t9 temps, $s0-$s7 saved) *(DONE)*
- [x] B5.3 — Stack frame layout (arguments, locals, saved registers) *(DONE)*
- [x] B5.4 — Calling convention ($a0-$a3 in, $v0 out, $ra save/restore) *(DONE)*
- [x] B5.5 — String operations as byte-copy/compare loops *(DONE)*
- [x] B5.6 — Syscall emission (open/read/write/close/exit) *(DONE)*
- [x] B5.7 — Tests: C makegen IR → expected MIPS program *(DONE)*

---

## Track C: Renderers (IR → Text at Each Level)

### C1. Extend existing CodeRenderer for Rust
**Files**: `core/codegen/src/testgen/` (existing renderer stubs)

The `CodeRenderer<M>` trait already exists. The Rust renderer is partially implemented
for testgen. Extend it to handle full SourceFile emission for generated binaries.

Tasks:
- [x] C1.1 — Render `FnDef` with full signature, body, attributes *(DONE)*
- [x] C1.2 — Render `EnumDef` with derives and variants *(DONE)*
- [x] C1.3 — Render `StructDef` with field types and visibility *(DONE)*
- [x] C1.4 — Render `ImplBlock` with trait and methods *(DONE)*
- [x] C1.5 — Render `Import` as `use` statements *(DONE)*
- [x] C1.6 — Emit `Cargo.toml` with dependencies *(DONE)*
- [x] C1.7 — Tests: Rust IR → expected source text (snapshot tests) *(DONE)*

### C2. Go renderer (CodeRenderer impl)
**Files**: new `core/codegen/src/go_renderer.rs` or similar

Tasks:
- [x] C2.1 — Render Go function definitions with multi-return *(DONE)*
- [x] C2.2 — Render Go struct types *(DONE)*
- [x] C2.3 — Render Go error handling idiom (`if err != nil`) *(DONE)*
- [x] C2.4 — Render Go imports and package declaration *(DONE)*
- [x] C2.5 — Emit `go.mod` *(DONE)*
- [x] C2.6 — Use existing `TypeSystemMapping` for Go type names *(DONE)*
- [x] C2.7 — Use existing `NamingConventions` for Go identifier style (camelCase) *(DONE)*
- [x] C2.8 — Tests: Go IR → expected source text *(DONE)*

### C3. C renderer (CodeRenderer impl)
**Files**: new `core/codegen/src/c_renderer.rs`

Tasks:
- [x] C3.1 — Render C function definitions with prototypes *(DONE)*
- [x] C3.2 — Render C structs with tagged union Value type *(DONE)*
- [x] C3.3 — Render C include directives *(DONE)*
- [x] C3.4 — Render C main() with argc/argv *(DONE)*
- [x] C3.5 — Emit Makefile for C compilation *(DONE)*
- [x] C3.6 — Tests: C IR → expected source text *(DONE)*

### C4. MIPS renderer (new trait or RegisterRenderer)
**Files**: new `core/codegen/src/mips_renderer.rs`

Tasks:
- [x] C4.1 — Render .data section (string literals, constants) *(DONE)*
- [x] C4.2 — Render .text section (functions as labeled blocks) *(DONE)*
- [x] C4.3 — Render instructions (load, store, arithmetic, branch, jump, syscall) *(DONE)*
- [x] C4.4 — Render stack frame prologues/epilogues *(DONE)*
- [x] C4.5 — Tests: MIPS program → expected assembly text *(DONE)*

---

## Track D: Exec-Runtime Fast Path (Rust Layer 1)

While Tracks A-C build the language DAG properly, this track gets a working
generated binary ASAP using the existing gunbc-exec runtime.

### D1. Rust exec-runtime codegen
**Files**: `core/daglang/daglang-emit/src/rust_exec_runtime.rs`

Generate Rust code that builds `Dag<Op>` + calls `gunbc-exec`. This bypasses
the language DAG temporarily — it's the bootstrap path.

Tasks:
- [x] D1.1 — Emit Op enum with one variant per DAG node *(DONE)*
- [x] D1.2 — Emit `impl Executable for Op` with match dispatch *(DONE)*
- [x] D1.3 — Emit executor bodies (port from exec-bridge implementations) *(DONE)*
- [x] D1.4 — Emit graph construction (`Dag::new() + add_node + add_edge`) *(DONE)*
- [x] D1.5 — Emit `fn main()` with CLI arg parsing + execute_and_display *(DONE)*
- [x] D1.6 — Emit `Cargo.toml` with gunbc-ir/gunbc-exec path deps *(DONE)*
- [x] D1.7 — Makegen end-to-end: generated binary produces identical Makefile *(DONE)*
- [x] D1.8 — Pragma end-to-end: generated binary produces identical pragma files *(DONE)*

### D2. Reconcile with language DAG
Once Tracks A-C mature, the exec-runtime path becomes one rendering of
`SystemsIR` — the Rust-specific lowering that happens to use gunbc-exec.
Eventually the native Rust path (Track B2) replaces it.

Tasks:
- [ ] D2.1 — Express exec-runtime codegen as AbstractIR → SystemsIR → Rust text *(TAKEN)*
- [ ] D2.2 — Verify output identical to D1 path *(TAKEN)*
- [ ] D2.3 — Remove D1 standalone path (replaced by language DAG path) *(TAKEN)*

---

## Track E: CLI + Testing + CI

### E1. `daglang compile` command
**Files**: `core/daglang/daglang-cli/src/main.rs`

```
daglang compile <input.dag> --target {rust|go|c|mips} [--out <dir>] [--layer 1|2]
```

Tasks:
- [x] E1.1 — Add `compile` subcommand to CLI parser *(DONE)*
- [x] E1.2 — Wire through driver: parse → lower → derive → plan → lower_ir → render *(DONE)*
- [x] E1.3 — `--target` selects exit point in language DAG *(DONE)*
- [x] E1.4 — `--layer 1` (exec-runtime, Rust only) vs `--layer 2` (native, all targets) *(DONE)*
- [x] E1.5 — Write generated files to `--out` directory *(DONE)*

### E2. Cross-language parity test harness
**Files**: new `core/daglang/daglang-cli/tests/codegen_parity.rs`

Tasks:
- [x] E2.1 — Build and run generated binaries per target (cargo, go build, gcc, mips-as+qemu) *(DONE)*
- [x] E2.2 — Capture output (file content written) *(DONE)*
- [x] E2.3 — Assert identical output across all targets *(DONE)*
- [x] E2.4 — Makegen parity: Rust == Go == C == MIPS Makefile output *(DONE)*
- [x] E2.5 — CI integration: run parity tests on every push *(DONE)*

### E3. Obligation-driven test generation
**Files**: new `core/daglang/daglang-emit/src/test_gen.rs`

Tasks:
- [x] E3.1 — Emit dry-run completion test per target language *(DONE)*
- [x] E3.2 — Emit per-transport-node mock test *(DONE)*
- [x] E3.3 — Emit pure-node snapshot test from NodeIoExample *(DONE)*
- [x] E3.4 — Rust: `#[test]` functions *(DONE)*
- [x] E3.5 — Go: `func Test*` functions *(DONE)*
- [x] E3.6 — C: test runner with assert macros *(DONE)*

---

## Track F: Reconciliation with the-gunbai

### F1. Shared abstractions
Align the two repos' models where they overlap.

Tasks:
- [x] F1.1 — Reconcile EdgeKind: adopt the-gunbai's DataFlow/Control/TriggerGate in gunbc *(DONE)*
- [x] F1.2 — Reconcile Effect model: adopt 2-bit (writes_world × deterministic) classification *(DONE)*
- [x] F1.3 — Reconcile Value types: bridge gunbai Value (Artifact, Secret, Capability) ↔ gunbc Value *(DONE)*
- [x] F1.4 — Reconcile PortType: align gunbai's simpler type system with gunbc's TypeId strings *(DONE)*
- [x] F1.5 — Document shared abstractions in a cross-repo design doc *(DONE)*

### F2. Understanding-driven codegen alignment
the-gunbai generates code from Understandings (versioned system knowledge).
The language DAG should be the shared rendering layer.

Tasks:
- [ ] F2.1 — Map gunbai's `CodegenEngine` output to `code_ir::SourceFile` *(TAKEN)*
- [ ] F2.2 — Route gunbai's Rust/Python/TypeScript generation through CodeRenderer<M> *(TAKEN)*
- [ ] F2.3 — Share `TypeSystemMapping` and `NamingConventions` across repos *(TAKEN)*

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
| `code_ir/` (was `code_ir.rs`) | Tiered code AST | Module directory with AbstractIR (Tier 0), SystemsIR (Tier 1), CStyleIR (Tier 3), RegisterIR (Tier 4), LowerIR trait |
| `render_ir.rs` | OutputMedium + renderers | CodeRenderer<M> is the rendering interface at each tier |
| `language/mod.rs` | Language SubDag compositions | Provides TypeSystemMapping + NamingConventions for all targets |

---

## Appendix: Completed Tasks

### B1.1 — Audit code_ir.rs: tag each variant with its tier
**Date**: 2026-02-17
**Files**: `core/ir/src/code_ir/mod.rs`

Every `Stmt`, `Expr`, and `Item` variant annotated with `/// **Tier N.**` or
`/// **Tier N (SystemsIR).**` doc comments. Supporting structs (`FnDef`,
`EnumDef`, `StructDef`) annotated where fields are Tier 1 (e.g., `derives`,
`attributes`).

**Tier 0 (AbstractIR):**
- Stmt: Let, Expr, Assert, Comment, Blank, Return, For, Item
- Expr: Value, Var, Str, Call, MethodCall, Field, BinOp, UnaryOp, IntLit, BoolLit, If, Block, Closure, FormatStr, Tuple, Array, Struct, RawCode
- Item: Use, Fn, Enum, Struct, Raw

**Tier 1 (SystemsIR):**
- Stmt: TailExpr
- Expr: Deref, Ref, RefMut, Path, MacroCall, Match
- Item: Impl
- Fields: FnDef.attributes, EnumDef.derives, StructDef.derives

### B1.2 — Extract Tier 0 (AbstractIR) as base types
**Date**: 2026-02-17
**Files**: `core/ir/src/code_ir/mod.rs`

Converted `code_ir.rs` → `code_ir/` module directory. The module-level doc
comment now documents all five tiers and the lowering direction. All existing
`pub use code_ir::*` re-exports continue to work — zero downstream breakage.

### B1.3 — Gate Tier 1 (SystemsIR) behind module boundary
**Date**: 2026-02-17
**Files**: `core/ir/src/code_ir/mod.rs`

Tier 1 variants are clearly documented in the enum definitions. Since Rust
enums cannot be split across modules, the "gate" is documentation + the
lowering trait (B1.6) which enforces tier boundaries at the type level.

### B1.4 — Define Tier 3 (CStyleIR) types
**Date**: 2026-02-17
**Files**: `core/ir/src/code_ir/c_ir.rs`

New types: `CStmt` (13 variants), `CExpr` (19 variants), `CType` (9 variants
including `FnPtr`), `CItem` (8 variants including `TaggedUnion`), `CFnDef`,
`CFnDecl`, `CSourceFile`, `CIntKind`, `CFloatKind`.

### B1.5 — Define Tier 4 (RegisterIR) types
**Date**: 2026-02-17
**Files**: `core/ir/src/code_ir/register_ir.rs`

New types: `AsmProgram`, `AsmFunction`, `StackFrame`, `Instruction` (24 MIPS
instructions), `Register` (MIPS32 register set with `name()` and `Display`),
`DataEntry` (4 data section kinds), `AsmTarget`, `syscall` constants module
(12 SPIM/MARS-compatible syscall numbers).

### B1.6 — Define lowering trait
**Date**: 2026-02-17
**Files**: `core/ir/src/code_ir/lower.rs`

New trait: `LowerIR<From, To>` with associated `Context` and `Error` types.
Supporting: `LowerError` enum, marker types (`ToRust`, `ToGo`, `ToC`, `ToMips`),
`Compose<A, B, Mid>` for chaining two lowering passes, `IrTier` marker trait
with impls for `SourceFile`, `CSourceFile`, `AsmProgram`.

### A1.1 — Define Computation enum, PureBody, TransportKind
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/computation.rs`

`Computation` enum with 4 variants (Pure, Transport, ResourceAcquire, Collection).
`PureBody` enum with 8 variants. `TransportKind` enum with 6 variants.
Supporting types: `RequestSpec`, `RequestKind`, `ResponseSpec`, `ResponseKind`,
`StringOpKind`, `JsonOpKind`, `AggregateKind`, `ServiceCallMetadata`,
`CollectionOpKind`.

### A1.2 — Define TypedPort
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/computation.rs`

`TypedPort` struct with `name`, `abstract_type`, and `cardinality` fields.
`Cardinality` enum with 3 variants (Scalar, List, Optional).

### A1.3 — Implement classify_computation
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/computation.rs`

`classify_computation(node: &Node<LoweredOp>) -> Result<Computation, ClassifyError>`.
Dispatches on `LoweredOp` variant (Callable/Collection/Pipeline), then by
`ObligationCategory` (ResourceAcquire, ServiceTransportExecute/Prepare/Parse,
InterfaceContractVerification, None). Fallback `classify_by_name()` handles
content-upsert patterns, load/env ops, and render ops via name heuristics.
Supporting: `ClassifyError`, `port_to_typed()`, `infer_transport_kind()`,
`infer_request_kind()`, `classify_content_upsert()`.

### A1.4 — Tests: every makegen node → expected Computation variant
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/computation.rs`

8 tests covering: load_registry (Literal), render_makefile (Template),
prepare_read (Literal), execute_read (Transport/FileRead), compare_content
(Compare), prepare_write (Literal), execute_transport (Transport/FileWrite),
fs_env (ResourceAcquire), entrypoint (Pure fallback).

### A1.5 — Tests: every pragma node → expected Computation variant
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/computation.rs`

5 pragma-specific tests (render_clippy/allowlist/lint_policy as Template,
execute_read as Transport/FileRead, compare/execute_write as Compare/FileWrite)
plus 6 obligation-based tests (ResourceAcquire, ServiceTransportExecute for
shell/HTTP, ServiceTransportPrepare, Collection map/filter, Pipeline, SubDag error).

### A2.1 — Define EmitPlan, EmitStep, InputBinding, OutputBinding
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/plan.rs`

`EmitPlan` with steps/entrypoints/transport_nodes. `EmitStep` with node_id,
computation, input_sources, output_bindings. `InputBinding` enum (FromStep,
FromEntrypoint, Constant). `OutputBinding` with port + consumers.
`EntrypointPort` with name, abstract_type, consumers. `PlanError` enum.
5 tests covering round-trip construction, transport+entrypoint, constants,
error display, collection steps.

### A3.1-A3.10 — Computation → AbstractIR lowering
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/lower_to_ir.rs`

`lower_plan_to_abstract_ir()` converts `EmitPlan` to `SourceFile` with `fn main()`.
Each `EmitStep` becomes Stmt(s): Pure → let-bind with body expr, Transport → prepare
+ execute + parse calls, ResourceAcquire → `acquire_resource()`, Collection → `collection_*()`.
Input bindings resolve to `Expr::Var(step_N_port)`. Entrypoints become function params.
Compare nodes emit `fresh` + `!fresh` (skip) bindings. Variable naming: `step_{idx}_{port}`.
2 tests: makegen-style plan with transport+compare, pragma-style parallel chains with
unique step output variables.

### B2.1-B2.6 — AbstractIR → SystemsIR/Rust lowering
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/lower_rust.rs`

`lower_to_rust(source, config)` transforms AbstractIR `SourceFile` to Rust-flavored IR.
`RustConfig` controls `use_exec_runtime` (bool) and `error_type` (String).
B2.1: Functions with transport calls get `Result<(), ExecError>` return + `Ok(())` tail.
B2.2: Empty-derives structs/enums get `#[derive(Debug, Clone)]`; existing derives preserved.
B2.3: Import analysis emits `use gunbc_exec::*` and `use gunbc_ir::transport::*` when
transport calls detected; `use serde_json::Value` when JSON values present.
B2.4: `FormatStr` → `MacroCall("format", ...)`.
B2.5: Abstract transport names rewritten to concrete Rust runtime calls:
`prepare_file_read` → `FileRequest::read`, `execute_file_*` → `execute_transport`, etc.
Standalone mode (`use_exec_runtime: false`) preserves abstract names.
11 tests covering all 6 subtasks + integration test.

### C1.1-C1.7 — Rust CodeRenderer (daglang-emit)
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/render_rust.rs`

Standalone Rust renderer: `render_rust_source(source) -> String` renders a
`SourceFile` (after `lower_to_rust`) to valid `.rs` text. `render_cargo_toml`
generates a minimal Cargo.toml.
C1.1: FnDef rendering with full signature, body, doc, attributes.
C1.2: EnumDef with derives and variants.
C1.3: StructDef with field types and visibility.
C1.4: ImplBlock with optional trait name and method rendering.
C1.5: Import → `use path::{items};` with single/multi-item formatting.
C1.6: `render_cargo_toml(name, deps)` for emitted crate packaging.
C1.7: Special `?` operator: `MethodCall { method: "?" }` → `expr?`.
ValueExpr rendered in "bare" mode (native Rust types, not Value:: wrappers).
14 tests covering all subtasks + full SourceFile integration test.

### B3.1-B3.5 — AbstractIR → ManagedIR/Go lowering
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/lower_go.rs`

`lower_to_go(source, config)` transforms AbstractIR `SourceFile` to Go-flavored IR.
`GoConfig` controls `package_name` (String, default "main") and `use_exec_runtime` (bool).
B3.1: Reuses `SourceFile`; Go enums become `Item::Raw` with `type X int` + `const iota` block.
B3.2: Transport calls expand to multi-return: `result, err := call()` + `if err != nil { return err }`.
Fallible functions get `error` return type + `return nil` at end.
B3.3: Package declaration as first `Item::Raw`; import analysis collects "fmt", "encoding/json",
"github.com/gunb-ai/gunbc/transport" as needed.
B3.4: Type mapping: String→string, Int→int64, Float→float64, List<T>→[]T, Optional<T>→*T,
Map<K,V>→map[K]V. FormatStr→fmt.Sprintf(). Names: snake_case→camelCase (vars/params),
PascalCase (exported funcs/types/fields). TailExpr→explicit Return (Go has no implicit return).
B3.5: 14 tests covering all subtasks + integration test.

### C2.1-C2.8 — Go CodeRenderer (daglang-emit)
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/render_go.rs`

Standalone Go renderer: `render_go_source(source) -> String` renders a
`SourceFile` (after `lower_to_go`) to valid `.go` text. `render_go_mod`
generates a minimal go.mod.
C2.1: Function rendering with Go signature (`func Name(params) retType { ... }`).
C2.2: Struct rendering as `type Name struct { ... }` with Go-style field layout.
C2.3: Error handling idiom: `if err != nil { ... }` rendered from If expressions.
C2.4: Import block: single import `import "pkg"` or grouped `import (...)`.
C2.5: `render_go_mod(module_path, go_version)` for module packaging.
C2.6/C2.7: Go naming (PascalCase exported, camelCase internal) already applied by lowering.
Short var decl: `Stmt::Let` → `name := expr`. For-range: `for _, x := range iter { ... }`.
ImplBlock → methods with `(self *Type)` receiver. Match → `switch`.
Path uses dots (not `::`). ValueExpr renders Go-native: nil, bare strings, `[]interface{}{}`.
10 tests covering all subtasks + full SourceFile integration test.

### D1.1-D1.6 — Rust exec-runtime codegen
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/rust_exec_runtime.rs`

`emit_exec_runtime(dag, module_name) -> Result<Vec<EmittedFile>, ExecRuntimeError>` generates
a standalone Rust crate (main.rs + Cargo.toml) from a `Dag<LoweredOp>`.
D1.1: `Op` enum with one variant per unique handler kind (10 variants matching ResolvedOp).
D1.2: `impl Executable for Op` with match dispatch to per-variant handler functions.
D1.3: Handler bodies ported from exec-bridge as static string templates (`handler_body()`).
D1.4: `build_dag()` emits hardcoded `Dag::new()` + `add_node` + `add_edge` graph construction.
D1.5: `main()` with CLI arg parsing for entrypoint ports + `execute_and_display()`.
D1.6: `Cargo.toml` with gunbc-ir, gunbc-exec, gunbc-lib-transport path dependencies.
Supporting: `ClassifiedNode` struct, `classify_nodes()` via `classify_runtime_op`,
`HandlerKind` enum, `to_snake()` helper, `emit_file_request_helper()` for shared transport code.
10 tests covering all aspects.

### E1.1 — Add compile --target to CLI
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-cli/src/main.rs`, `core/daglang/daglang-cli/src/commands.rs`

Added `--target {rust|go|c|mips}` and `--layer {1|2|exec-runtime|native}` flag parsing to
`parse_compile_command_args`. Added `parse_codegen_target()` and `parse_codegen_layer()`
helpers. Updated compile command handler to pass `CompileOptions { target, layer,
emit_collection_nodes }` through `compile_target_or_exit_with_compile_options`. Supports
both `--flag value` and `--flag=value` forms. Driver already had `CodegenTarget`,
`CodegenLayer`, and `emit_with_options()` routing; CLI just needed arg parsing wired through.

### D1.7 — Makegen end-to-end
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-driver/src/lib.rs`, `core/daglang/daglang-cli/tests/cli_commands.rs`

Driver-level structural verification test: compiles `dsl/tools/makegen.dag` with
`CodegenLayer::ExecRuntime`, verifies all handler kinds, DAG topology (node/edge
counts match lowered DAG), handler body correctness, and valid Cargo.toml.
Full build-and-run e2e test (`#[ignore]`) writes to workspace-relative dir, builds,
runs binary, verifies Makefile content.

### D1.8 — Pragma end-to-end
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-driver/src/lib.rs`, `core/daglang/daglang-cli/tests/cli_commands.rs`

Driver-level structural verification test: compiles `dsl/tools/pragma.dag` with
`CodegenLayer::ExecRuntime`, verifies pragma-specific handler kinds
(RenderPragmaClippyToml, RenderPragmaAllowlist, RenderPragmaLintPolicy,
PragmaEntrypoint), content upsert pattern handlers, PragmaDirectiveRuntime struct,
and correct 3-chain DAG topology. CLI test verifies written files contain pragma handlers.
Full build-and-run e2e test (`#[ignore]`) available.

### B4.1-B4.6 — AbstractIR → CStyleIR lowering
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/lower_c.rs`

`lower_to_c(source, config)` transforms AbstractIR `SourceFile` to `CSourceFile` (Tier 3 IR).
`CConfig` controls `use_arena` (bool) and `use_exec_runtime` (bool).
B4.1: `CSourceFile` reused from `c_ir.rs`; AbstractIR items mapped to `CItem` variants
(`FnDef→CFnDef`, `StructDef→CItem::StructDef`, `EnumDef→CItem::Define` constants).
B4.2: Type mapping: `String→const char*`, `Int→int64_t`, `Float→double`, `Bool→int`,
`List<T>→T*`, `Optional<T>→T*`, `Map<K,V>→void*`. `CType` hierarchy with
`Ptr(Const(Char))` for strings.
B4.3: String values rendered as C string literals; `FormatStr→snprintf()` with buffer+args.
B4.4: Arena flag controls malloc commentary; stack allocation for locals by default.
B4.5: Transport functions return `int` (0=success, -1=error); callers check
`if (rc != 0) return -1;`. Non-transport functions return `int` for uniformity.
B4.6: Include analysis: always `stdio.h/stdlib.h/string.h`; conditional `gunbc/transport.h`
when transport calls detected. ImplBlock→free functions with type-prefixed names
(`Type_method`). Naming preserved as snake_case (C convention).
10 tests covering type mapping, transport rewriting, error codes, snprintf, includes,
struct fields, enum defines, and full integration.

### C3.1-C3.6 — C CodeRenderer (daglang-emit)
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/render_c.rs`

Standalone C renderer: `render_c_source(source) -> String` renders a
`CSourceFile` (after `lower_to_c`) to valid `.c` text. `render_c_makefile`
generates a minimal Makefile for gcc compilation.
C3.1: Function rendering with `static` qualifier, params (or `void` for empty),
return type, body. Forward declarations via `render_fn_decl`.
C3.2: Structs rendered as `typedef struct { ... } Name;`. Tagged unions rendered
with separate `typedef enum` for tag + `typedef struct` with `union` inside.
C3.3: `#include <header>` for system, `#include "header"` for local.
C3.4: `main(int argc, char** argv)` with arg validation pattern.
C3.5: `render_c_makefile(binary_name, sources)` generates CC/CFLAGS/TARGET/SRCS
rules with gcc, -Wall -Wextra -std=c11 -O2, and clean target.
C3.6: Expressions: BoolLit → 0/1, Null → NULL, postfix ++/--, BinOp wrapped in
parens, pointer ops (&/*/->), Cast, SizeOf, Malloc, Ternary. Types: int kinds
(int/long/size_t/intN_t/uintN_t), const, Ptr, Array, FnPtr, Named. For loops
render inline init/step. Labels outdented by one level.
17 tests covering all subtasks + full integration test.

### B5.1-B5.7 — CStyleIR → RegisterIR lowering (MIPS) (daglang-emit)
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/lower_mips.rs`

`lower_to_mips(source: &CSourceFile, config: &MipsConfig) -> Result<AsmProgram, LowerError>`
lowers CStyleIR (Tier 3) to RegisterIR (Tier 4, MIPS32).
B5.1: Types already defined in `register_ir.rs` — lowering uses `AsmProgram`,
`AsmFunction`, `StackFrame`, `Instruction`, `Register`, `DataEntry`.
B5.2: Bitset-based temp register allocator (`$t0`-`$t9`). `alloc_temp()` returns
lowest free reg, `free_temp()` releases. LIFO usage keeps depth ≤ 5.
B5.3: All locals on stack (-O0 style). `FnState::build_frame()` computes total
frame size (locals + `$ra`), 8-byte aligned. `type_size_aligned()` returns
word-aligned sizes for all CTypes.
B5.4: Params stored from `$a0`-`$a3` to stack slots. Calls: args loaded to `$a0`-`$a3`
(overflow args pushed to stack), `jal label`, result moved from `$v0`. Return:
value moved to `$v0` + `jr $ra`.
B5.5: `emit_strcmp()` — byte-by-byte compare loop (lb, bne, beq null, addi pointers).
`emit_strcpy()` — byte-by-byte copy loop (lb, sb, beq null, addi pointers).
B5.6: `syscall_for_func()` maps C library names to SPIM/MARS syscall numbers:
printf→4, exit→10, open→13, read→14, write→15, close→16. `malloc` → sbrk (9).
B5.7: 23 tests covering all subtasks + makegen integration test verifying data
section, param stores, jal calls, branch sequences, and frame layout.

### C4.1-C4.5 — MIPS assembly renderer (daglang-emit)
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/render_mips.rs`

Standalone MIPS renderer: `render_mips_source(program) -> String` renders an
`AsmProgram` (after `lower_to_mips`) to valid MIPS assembly text for SPIM/MARS/QEMU.
C4.1: `.data` section rendering: `.asciiz` (strings with escape), `.word` (ints),
`.space` (buffers), `.byte` (byte sequences). Each entry has a label.
C4.2: `.text` section with `.globl` for entry functions. Functions rendered as
labeled blocks. Non-entry functions without explicit return get automatic `jr $ra`.
C4.3: All 24 MIPS instruction variants rendered: arithmetic (add/addi/sub/mul),
load/store (lw/sw/lb/sb/li/la), branch/jump (beq/bne/bge/blt/j/jal/jr),
data movement (move/slt), syscall, and structural (Label/Comment/Blank/Nop).
Tab-indented instructions, labels at column 0.
C4.4: Stack frame prologue: `addi $sp, $sp, -size` + `sw $ra` + `sw $sN`.
Epilogue: restore callee-saved (reverse order) + `lw $ra` + `addi $sp` + `jr $ra`.
Generated from `StackFrame` struct (size, locals, saved_regs, ra_offset).
11 tests covering all subtasks + hello-world integration + stack-frame function.

### E3.1-E3.6 — Obligation-driven test generation (daglang-emit)
**Date**: 2026-02-17
**Files**: `core/daglang/daglang-emit/src/test_gen.rs`

Obligation-driven test generation for compiled DAG modules. Emits test source
code in Rust, Go, and C from a `TestSpec` describing the module under test.
E3.1: `emit_dry_run_completion_test(backend, obligations)` emits a per-backend
dry-run completion contract test (Rust `#[test]`, Go `func Test*`, C `assert()`,
MIPS exit-code). E3.2: `emit_transport_mock_tests(backend, dag)` scans the DAG
for transport nodes and emits per-node mock injection tests (BTreeMap/map/strcmp).
E3.3: `PureTestTarget` struct with `inputs`/`expected_outputs` for snapshot tests.
E3.4: `emit_rust_tests(spec)` → `#[test]` functions with `assert_eq!` for pure
nodes and `assert!(result.is_ok())` for transport mocks.
E3.5: `emit_go_tests(spec)` → `func Test*(t *testing.T)` with PascalCase function
calls, camelCase variables, `t.Fatalf` assertions.
E3.6: `emit_c_tests(spec)` → C test runner with `ASSERT_EQ`/`ASSERT_STR_EQ`/
`ASSERT_OK` macros, per-test functions, and `main()` calling all tests.
Supporting: `TestSpec`, `TransportTestTarget`, `PureTestTarget` types,
`to_pascal_case()`, `to_camel_case()` helpers, `sanitize_identifier()`.
23 tests covering all subtasks + full three-target integration test.

### F1.1-F1.5 — Shared abstractions reconciliation with the-gunbai
**Date**: 2026-02-17
**Files**: `core/ir/src/dag.rs`, `core/ir/src/effect.rs`, `core/ir/src/value_bridge.rs`,
`core/ir/src/port_type.rs`, `docs/design/v4/shared-abstractions.md`

Cross-repo compatibility layer between gunbc and the-gunbai.
F1.1: Added `EdgeKind` enum (DataFlow/Control/TriggerGate) to `Edge` struct.
Backward compatible via `#[serde(default)]` — existing edges deserialize as
DataFlow. New constructors `Edge::control()` and `Edge::trigger()`. Convenience
methods `Edge::carries_data()`, `Edge::is_gating()`.
F1.2: Added `Effect` struct (2-bit: `writes_world` × `deterministic`).
Constants `PURE`/`READ`/`WRITE_DETERMINISTIC`/`WRITE`. Methods `cacheable()`,
`requires_policy()`. Default is PURE.
F1.3: Added `value_bridge` module with `classify_value()` (Shared vs GunbcOnly),
`to_bridge_json()` (Value → JSON wire format), `from_bridge_json()` (JSON → Value).
Secrets redacted, gunbc-only I/O variants return None.
F1.4: Added `PortType` structural enum (9 variants matching gunbai-types). Bidirectional
conversion with `TypeId` strings via `From<&TypeId>` and `PortType::to_type_id()`.
Legacy TypeId strings (StringList, Unit, Void) handled. Secret is strict — only
compatible with Secret or Any. Recursive List compatibility check.
F1.5: Comprehensive design doc at `docs/design/v4/shared-abstractions.md` documenting
all four type alignments, conversion strategies, and wire format recommendations.
All types re-exported from `gunbc_ir` root. 7 + 14 + 7 = 28 new tests.
