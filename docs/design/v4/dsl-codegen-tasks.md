# DSL Codegen: Parallelizable Task Breakdown

Each task is a self-contained work unit with:
- **Owns**: files this task creates or modifies (exclusive — no other task touches these)
- **Reads** (not modifies): files this task needs to reference
- **Produces**: the public interface downstream tasks depend on
- **Blocked by**: tasks that must complete first (`—` = no blockers, start immediately)
- **Tests**: concrete assertions that define "done"

---

## Wave 1 — No Dependencies (start all immediately)

### Task 1: Computation Types
Define the target-independent computation model — what each DAG node *does*,
not how it's expressed in any language.

- **Owns**: `core/daglang/daglang-emit/src/computation.rs` (new)
- **Reads**: `core/daglang/daglang-lower/src/lib.rs` (LoweredOp, ServiceCallMetadata, CallableKind, ObligationCategory)
- **Blocked by**: —

**Produces**:
```rust
// computation.rs — all types pub

/// What a node computes, target-independently.
pub enum Computation {
    Pure { inputs: Vec<TypedPort>, outputs: Vec<TypedPort>, body: PureBody },
    Transport { kind: TransportKind, inputs: Vec<TypedPort>, outputs: Vec<TypedPort> },
    ResourceAcquire { handle_type: String, handle_value: String },
    Collection { kind: CollectionOpKind, element_type: String },
}

pub struct TypedPort {
    pub name: String,
    pub value_type: ValueType,
    pub is_list: bool,
    pub optional: bool,
}

pub enum ValueType {
    Str, Bool, Int, Json, TransportRequest, TransportResponse, Skipped, Any,
}

pub enum PureBody {
    Literal(serde_json::Value),
    Template { pattern: String, variables: Vec<String> },
    StringOp(StringOpKind),
    JsonOp(JsonOpKind),
    Compare { left: String, right: String },
    Conditional { condition: String, then_port: String, else_port: Option<String> },
    Aggregate { inputs: Vec<String>, strategy: AggregateKind },
    ServiceCall(ServiceCallMetadata),
}

pub enum TransportKind { FileRead, FileWrite, FileExists, ShellExec, HttpRequest }
pub enum StringOpKind { Concat, Split, Join, Filter, Map, Trim }
pub enum JsonOpKind { Parse, Extract, Serialize }
pub enum AggregateKind { AllSucceeded, AnySucceeded, Count, Collect }

/// Classify a lowered node into its Computation.
pub fn classify_computation(
    node: &Node<LoweredOp>,
) -> Result<Computation, ComputationError>;

pub struct ComputationError { pub node_id: String, pub reason: String }
```

**Tests** (in `computation.rs` or `tests/` within daglang-emit):
- [ ] `classify_makegen_load_registry` → `Pure(Literal(json!({...})))`
- [ ] `classify_makegen_render_makefile` → `Pure(Template{...})`
- [ ] `classify_makegen_prepare_read` → `Transport(FileRead, ...)`
- [ ] `classify_makegen_execute_read` → `Transport(FileRead, ...)`
- [ ] `classify_makegen_compare_content` → `Pure(Compare{...})`
- [ ] `classify_makegen_prepare_write` → `Transport(FileWrite, ...)`
- [ ] `classify_makegen_execute_transport` → `Transport(FileWrite, ...)`
- [ ] `classify_unknown_module` → `Err(ComputationError{...})`
- [ ] All 10 makegen nodes classify without error
- [ ] `cargo test -p daglang-emit` passes
- [ ] `cargo clippy -p daglang-emit -- -D warnings` clean

---

### Task 2: AbstractIR Core (factor code_ir)
Separate target-agnostic constructs from Rust-specific ones in code_ir.rs.
The abstract core becomes the universal top of the language DAG.

- **Owns**: `core/ir/src/code_ir.rs` (refactor in place)
- **Reads**: nothing new
- **Blocked by**: —

**Produces**: The same types, reorganized into clearly marked tiers within code_ir.rs.
All existing downstream code continues to compile unchanged.

```rust
// code_ir.rs — reorganized sections

// ──── Tier 0: AbstractIR (universal, no language features) ────
// Stmt: Let, Expr, If, For, Return, Comment, Blank
// Expr: Var, Str, IntLit, BoolLit, Call, MethodCall, Field, BinOp,
//       UnaryOp, Array, Block, If, Match, FormatStr, Tuple
// Item: FnDef, StructDef, EnumDef
// Assert: Eq, True, NonEmpty, Contains
// Import, MatchArm, TestFile, TestSection, TestFn, HelperFn, SourceFile

// ──── Tier 1: SystemsIR (Rust/C++ extensions) ────
// Expr: Deref, Ref, RefMut, MacroCall, Path, Closure, RawCode
// Stmt: TailExpr
// Item: ImplBlock
// EnumDef.derives, StructDef.derives

// pub fn is_abstract(stmt: &Stmt) -> bool;
// pub fn is_abstract_expr(expr: &Expr) -> bool;
```

**Tests**:
- [ ] `cargo test --workspace` passes (zero regressions — this is a refactor, not a rewrite)
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `is_abstract(Stmt::Let{..})` → true
- [ ] `is_abstract(Stmt::TailExpr{..})` → false
- [ ] `is_abstract_expr(Expr::Call{..})` → true
- [ ] `is_abstract_expr(Expr::Deref{..})` → false
- [ ] `is_abstract_expr(Expr::MacroCall{..})` → false
- [ ] All existing testgen codegen tests still pass

---

### Task 3: Exec-Runtime Makegen (fast path)
Generate a working Rust binary for makegen that uses gunbc-exec as runtime.
This bypasses the language DAG — it's the bootstrap proof that generated code works.

- **Owns**: `core/daglang/daglang-emit/src/rust_exec_runtime.rs` (new)
- **Reads**: `core/daglang/daglang-exec-bridge/src/lib.rs` (reference implementations), `core/daglang/daglang-lower/src/lib.rs` (LoweredOp), `core/ir/` (Dag, Node, Port, Edge)
- **Blocked by**: —

**Produces**:
```rust
// rust_exec_runtime.rs

/// Generate a complete Rust crate from a lowered DAG that uses gunbc-exec as runtime.
pub fn emit_exec_runtime_crate(
    dag: &Dag<LoweredOp>,
    artifacts: &DerivedArtifacts,
    config: &ExecRuntimeConfig,
) -> Result<EmissionBundle, EmitError>;

pub struct ExecRuntimeConfig {
    pub binary_name: String,        // "makegen"
    pub output_dir: String,         // "target/generated/makegen"
    pub workspace_root: String,     // for path deps in Cargo.toml
}
```

The `EmissionBundle` must contain at minimum:
- `Cargo.toml` with path deps to gunbc-ir, gunbc-exec, gunbc-lib-transport
- `src/main.rs` with: Op enum, `impl Executable for Op`, `fn build_graph()`, `fn main()`

**Tests**:
- [ ] `emit_exec_runtime_crate` for makegen DAG returns Ok with 2+ files
- [ ] Generated `Cargo.toml` contains `[dependencies]` with gunbc-ir, gunbc-exec
- [ ] Generated `src/main.rs` contains `enum Op` with correct variant count
- [ ] Generated `src/main.rs` contains `impl Executable for Op`
- [ ] Generated `src/main.rs` contains `fn build_graph()`
- [ ] Generated `src/main.rs` contains `fn main()`
- [ ] **Integration**: write generated files to tempdir, `cargo build` succeeds
- [ ] **Integration**: run generated binary, output Makefile matches hand-built binary output
- [ ] `cargo test -p daglang-emit` passes
- [ ] `cargo clippy -p daglang-emit -- -D warnings` clean

---

## Wave 2 — Depends on Wave 1

### Task 4: EmitPlan Builder
Build a topo-ordered execution plan from a lowered DAG + Computation classifications.
This is the shared data structure all code generation paths consume.

- **Owns**: `core/daglang/daglang-emit/src/plan.rs` (new)
- **Reads**: `computation.rs` (from Task 1), `core/ir/` (Dag, Edge, topo_sort)
- **Blocked by**: Task 1

**Produces**:
```rust
// plan.rs

pub struct EmitPlan {
    pub steps: Vec<EmitStep>,
    pub entrypoints: Vec<EntrypointPort>,
    pub transport_node_ids: Vec<String>,
}

pub struct EmitStep {
    pub node_id: String,
    pub computation: Computation,
    pub inputs: Vec<InputBinding>,
    pub outputs: Vec<OutputBinding>,
}

pub enum InputBinding {
    /// Value comes from a previous step's output port
    FromStep { step_index: usize, port_name: String },
    /// Value comes from CLI arg / function parameter
    FromEntrypoint { port_name: String, value_type: ValueType },
    /// Hardcoded constant
    Constant(serde_json::Value),
}

pub struct OutputBinding {
    pub port_name: String,
    pub value_type: ValueType,
    /// Local variable name for generated code (e.g., "step_3_response")
    pub var_name: String,
}

pub struct EntrypointPort {
    pub node_id: String,
    pub port_name: String,
    pub value_type: ValueType,
}

pub fn build_emit_plan(
    dag: &Dag<LoweredOp>,
    artifacts: &DerivedArtifacts,
) -> Result<EmitPlan, PlanError>;

pub struct PlanError { pub reason: String }
```

**Tests**:
- [ ] Makegen DAG → plan with 10 steps in valid topo order
- [ ] Step 0 (load_registry) has no `FromStep` inputs
- [ ] Step for render_makefile has `FromStep { step_index: <load_registry>, port: "registry" }`
- [ ] `entrypoints` contains "path" port
- [ ] `transport_node_ids` contains execute_read and execute_transport nodes
- [ ] Each step's `var_name` is unique
- [ ] Pragma DAG → plan with correct step count and parallel chain ordering
- [ ] `cargo test -p daglang-emit` passes, clippy clean

---

### Task 5: CStyleIR Types
Define the C-level intermediate representation. This is what AbstractIR
lowers to before reaching MIPS.

- **Owns**: `core/ir/src/c_ir.rs` (new), add `pub mod c_ir;` to `core/ir/src/lib.rs`
- **Reads**: `core/ir/src/code_ir.rs` (AbstractIR types for reference)
- **Blocked by**: Task 2 (needs AbstractIR tier markers to know what's Tier 0)

**Produces**:
```rust
// c_ir.rs

/// A C source file (one .c + one .h)
pub struct CSourceFile {
    pub includes: Vec<String>,          // #include <stdio.h>
    pub typedefs: Vec<CTypeDef>,
    pub functions: Vec<CFunction>,
    pub main: Option<CFunction>,
    pub string_literals: Vec<(String, String)>,  // (label, value) for .rodata
}

pub enum CType {
    Void, Char, Int, Int64, SizeT, Bool,
    Pointer(Box<CType>),                // char*
    Array(Box<CType>, Option<usize>),   // char[256]
    Struct(String),                      // struct Value
    FunctionPointer { params: Vec<CType>, ret: Box<CType> },
}

pub struct CTypeDef {
    pub name: String,
    pub kind: CTypeDefKind,
}

pub enum CTypeDefKind {
    Struct { fields: Vec<(String, CType)> },
    Enum { variants: Vec<(String, Option<i64>)> },
    TaggedUnion { tag_name: String, tag_type: String, variants: Vec<(String, CType)> },
    Alias(CType),
}

pub struct CFunction {
    pub name: String,
    pub params: Vec<(String, CType)>,
    pub return_type: CType,
    pub body: Vec<CStmt>,
    pub is_static: bool,
}

pub enum CStmt {
    VarDecl { name: String, ty: CType, init: Option<CExpr> },
    Assign { target: CExpr, value: CExpr },
    If { cond: CExpr, then_body: Vec<CStmt>, else_body: Option<Vec<CStmt>> },
    For { init: Box<CStmt>, cond: CExpr, step: CExpr, body: Vec<CStmt> },
    While { cond: CExpr, body: Vec<CStmt> },
    Return(Option<CExpr>),
    Expr(CExpr),
    Comment(String),
    Blank,
    Label(String),
    Goto(String),
}

pub enum CExpr {
    Var(String),
    IntLit(i64),
    StrLit(String),                    // "hello" (references string_literals label)
    CharLit(char),
    BoolLit(bool),
    Null,
    Call { func: String, args: Vec<CExpr> },
    BinOp { left: Box<CExpr>, op: CBinOp, right: Box<CExpr> },
    UnaryOp { op: CUnaryOp, expr: Box<CExpr> },
    Field { expr: Box<CExpr>, field: String },      // expr.field
    Arrow { expr: Box<CExpr>, field: String },      // expr->field
    Index { expr: Box<CExpr>, index: Box<CExpr> },  // expr[index]
    Deref(Box<CExpr>),                 // *expr
    AddrOf(Box<CExpr>),               // &expr
    Cast { ty: CType, expr: Box<CExpr> },
    SizeOf(CType),
    Ternary { cond: Box<CExpr>, then_expr: Box<CExpr>, else_expr: Box<CExpr> },
}

pub enum CBinOp { Add, Sub, Mul, Div, Mod, Eq, Ne, Lt, Le, Gt, Ge, And, Or, BitAnd, BitOr }
pub enum CUnaryOp { Neg, Not, BitNot }
```

**Tests**:
- [ ] `CSourceFile` can represent a minimal C program (main + printf)
- [ ] TaggedUnion can represent a Value type (tag enum + union of payloads)
- [ ] `CFunction` can represent a function with local variables, if/else, return
- [ ] All types derive Debug, Clone
- [ ] `cargo test --workspace` passes, `cargo clippy --all-targets -- -D warnings` clean

---

### Task 6: RegisterIR Types (MIPS)
Define the register-level representation for MIPS assembly emission.

- **Owns**: `core/ir/src/register_ir.rs` (new), add `pub mod register_ir;` to `core/ir/src/lib.rs`
- **Reads**: nothing (standalone type definitions)
- **Blocked by**: Task 2 (coordinate lib.rs module additions)

**Produces**:
```rust
// register_ir.rs

pub struct MipsProgram {
    pub data_section: Vec<DataEntry>,
    pub text_section: Vec<MipsFunction>,
    pub entry_label: String,              // "_start" or "main"
}

pub struct DataEntry {
    pub label: String,
    pub directive: DataDirective,
}

pub enum DataDirective {
    Asciiz(String),                       // .asciiz "hello"
    Word(Vec<i32>),                       // .word 1, 2, 3
    Space(usize),                         // .space 4096
    Byte(Vec<u8>),                        // .byte 0, 1, 2
}

pub struct MipsFunction {
    pub label: String,
    pub instructions: Vec<MipsInst>,
    pub frame_size: usize,                // bytes for stack frame
    pub saved_regs: Vec<Register>,        // callee-saved registers used
}

pub enum MipsInst {
    // Arithmetic
    Add(Register, Register, Register),    // add $d, $s, $t
    Addi(Register, Register, i16),        // addi $d, $s, imm
    Sub(Register, Register, Register),
    Mul(Register, Register, Register),
    // Memory
    Lw(Register, i16, Register),          // lw $t, offset($s)
    Sw(Register, i16, Register),          // sw $t, offset($s)
    La(Register, String),                 // la $t, label
    Li(Register, i32),                    // li $t, immediate
    Lb(Register, i16, Register),          // lb $t, offset($s)
    Sb(Register, i16, Register),          // sb $t, offset($s)
    // Control flow
    Beq(Register, Register, String),      // beq $s, $t, label
    Bne(Register, Register, String),      // bne $s, $t, label
    Bge(Register, Register, String),
    Blt(Register, Register, String),
    J(String),                            // j label
    Jal(String),                          // jal label
    Jr(Register),                         // jr $ra
    // Move
    Move(Register, Register),             // move $d, $s
    // Syscall
    Syscall,
    // Pseudo
    Nop,
    Comment(String),
    Label(String),                        // internal label (not function entry)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    Zero,                                 // $zero (always 0)
    V0, V1,                              // return values
    A0, A1, A2, A3,                      // arguments
    T0, T1, T2, T3, T4, T5, T6, T7,     // temporaries (caller-saved)
    S0, S1, S2, S3, S4, S5, S6, S7,     // saved (callee-saved)
    T8, T9,                              // more temporaries
    Sp,                                   // stack pointer
    Fp,                                   // frame pointer
    Ra,                                   // return address
}

impl Register {
    pub fn name(&self) -> &'static str;  // "$zero", "$v0", "$a0", etc.
}

impl std::fmt::Display for MipsInst {
    // Renders as MIPS assembly text: "add $t0, $t1, $t2"
}
```

**Tests**:
- [ ] `MipsProgram` can represent a minimal program (li $v0,10 + syscall = exit)
- [ ] `MipsFunction` can represent a function with stack frame setup/teardown
- [ ] `MipsInst::Display` renders correct assembly syntax for each instruction variant
- [ ] `Register::name()` returns correct MIPS register names
- [ ] `DataEntry` can represent string literal in .data section
- [ ] All types derive Debug, Clone
- [ ] `cargo test --workspace` passes, clippy clean

---

### Task 7: Go IR Extensions
Define Go-specific code_ir constructs that extend AbstractIR.

- **Owns**: `core/ir/src/go_ir.rs` (new), add `pub mod go_ir;` to `core/ir/src/lib.rs`
- **Reads**: `core/ir/src/code_ir.rs` (AbstractIR types)
- **Blocked by**: Task 2 (coordinate lib.rs module additions)

**Produces**:
```rust
// go_ir.rs

pub struct GoSourceFile {
    pub package: String,                  // "main"
    pub imports: Vec<GoImport>,
    pub functions: Vec<GoFuncDef>,
    pub types: Vec<GoTypeDef>,
    pub main: Option<GoFuncDef>,
}

pub struct GoImport {
    pub path: String,                     // "os", "fmt", "os/exec"
    pub alias: Option<String>,
}

pub struct GoFuncDef {
    pub name: String,
    pub params: Vec<(String, GoType)>,
    pub returns: Vec<GoType>,             // Go multi-return
    pub body: Vec<GoStmt>,
    pub doc: Vec<String>,
    pub is_exported: bool,                // Uppercase first letter
}

pub enum GoType {
    String, Bool, Int, Int64, Float64,
    Byte,
    Slice(Box<GoType>),                   // []T
    Map(Box<GoType>, Box<GoType>),        // map[K]V
    Interface(String),                    // interface name
    Struct(String),                       // struct name
    Pointer(Box<GoType>),                 // *T
    Error,                                // error interface
    Any,                                  // interface{}
}

pub struct GoTypeDef {
    pub name: String,
    pub kind: GoTypeDefKind,
}

pub enum GoTypeDefKind {
    Struct { fields: Vec<(String, GoType, Option<String>)> }, // (name, type, tag)
    Alias(GoType),
}

pub enum GoStmt {
    VarDecl { name: String, ty: Option<GoType>, init: Option<GoExpr> },
    ShortDecl { names: Vec<String>, exprs: Vec<GoExpr> },     // x, err := f()
    Assign { target: GoExpr, value: GoExpr },
    If { init: Option<Box<GoStmt>>, cond: GoExpr, body: Vec<GoStmt>, else_body: Option<Vec<GoStmt>> },
    For { init: Option<Box<GoStmt>>, cond: Option<GoExpr>, post: Option<Box<GoStmt>>, body: Vec<GoStmt> },
    ForRange { key: Option<String>, value: Option<String>, iter: GoExpr, body: Vec<GoStmt> },
    Return(Vec<GoExpr>),
    Expr(GoExpr),
    Comment(String),
    Blank,
    Defer(GoExpr),
}

pub enum GoExpr {
    Var(String),
    StrLit(String),
    IntLit(i64),
    BoolLit(bool),
    Nil,
    Call { func: Box<GoExpr>, args: Vec<GoExpr> },
    MethodCall { receiver: Box<GoExpr>, method: String, args: Vec<GoExpr> },
    Field(Box<GoExpr>, String),
    Index { expr: Box<GoExpr>, index: Box<GoExpr> },
    BinOp { left: Box<GoExpr>, op: String, right: Box<GoExpr> },
    UnaryOp { op: String, expr: Box<GoExpr> },
    Selector(Vec<String>),               // pkg.Func
    CompositeLit { ty: GoType, fields: Vec<(String, GoExpr)> },
    SliceLit(Vec<GoExpr>),
    MapLit(Vec<(GoExpr, GoExpr)>),
    FormatStr { verb: String, args: Vec<GoExpr> },  // fmt.Sprintf
    TypeAssert { expr: Box<GoExpr>, ty: GoType },
}
```

**Tests**:
- [ ] `GoSourceFile` can represent a minimal Go program (package main, func main, fmt.Println)
- [ ] `GoFuncDef` can represent multi-return function (result, error)
- [ ] `GoStmt::ShortDecl` can represent `x, err := someFunc()`
- [ ] `GoStmt::ForRange` can represent `for _, item := range items {}`
- [ ] All types derive Debug, Clone
- [ ] `cargo test --workspace` passes, clippy clean

---

## Wave 3 — Depends on Wave 1+2

### Task 8: Computation → AbstractIR Lowering
Convert an EmitPlan into target-agnostic code_ir constructs.
This is the bridge from "what" to "how" — still no language-specific features.

- **Owns**: `core/daglang/daglang-emit/src/lower_to_ir.rs` (new)
- **Reads**: `computation.rs` (Task 1), `plan.rs` (Task 4), `core/ir/src/code_ir.rs` (Task 2)
- **Blocked by**: Task 1, Task 2, Task 4

**Produces**:
```rust
// lower_to_ir.rs

use crate::plan::EmitPlan;
use gunbc_ir::code_ir::SourceFile;

/// Lower an EmitPlan into target-agnostic code_ir.
/// The resulting SourceFile uses only AbstractIR (Tier 0) constructs.
pub fn lower_plan_to_abstract_ir(plan: &EmitPlan) -> Result<SourceFile, LowerError>;

pub struct LowerError { pub step_index: usize, pub reason: String }
```

The generated SourceFile contains:
- A `fn main()` (or `fn run()`) with one Stmt sequence per EmitStep
- Each step's inputs bound via variable references to previous step outputs
- Entrypoints become function parameters
- Transport steps become `Call` to abstract transport function names
- Pure steps inline their computation as expressions
- Conditional steps become `If` blocks

**Tests**:
- [ ] Makegen plan → SourceFile with `fn main()` containing 10 step sequences
- [ ] Step 1 (LoadRegistry) → `Let { name: "registry", expr: Call("literal_json", ...) }`
- [ ] Step 2 (RenderMakefile) → `Let { name: "makefile_content", expr: Call("render_template", ...) }`
- [ ] Transport step → `Let { name: "response", expr: Call("execute_file_transport", ...) }`
- [ ] Compare step → `Let { name: "fresh", expr: BinOp(==, ...) }`
- [ ] Conditional write → `If { cond: UnaryOp("!", Var("fresh")), ... }`
- [ ] All expressions use only AbstractIR constructs (`is_abstract()` → true for all)
- [ ] `cargo test -p daglang-emit` passes, clippy clean

---

### Task 9: AbstractIR → Rust Lowering
Add Rust-specific constructs to AbstractIR output: ownership, Result, derives,
use statements, Cargo.toml.

- **Owns**: `core/daglang/daglang-emit/src/lower_rust.rs` (new)
- **Reads**: `core/ir/src/code_ir.rs` (SourceFile), `lower_to_ir.rs` (Task 8 output)
- **Blocked by**: Task 2, Task 8

**Produces**:
```rust
// lower_rust.rs

use gunbc_ir::code_ir::SourceFile;

pub struct RustCrateOutput {
    pub source_file: SourceFile,          // SourceFile with Rust-specific constructs added
    pub cargo_toml: String,               // Generated Cargo.toml content
    pub module_files: Vec<(String, SourceFile)>,  // Additional .rs files if needed
}

pub struct RustLowerConfig {
    pub crate_name: String,
    pub use_exec_runtime: bool,           // true = Layer 1, false = Layer 2 (standalone)
    pub workspace_root: Option<String>,   // for path deps
}

/// Lower AbstractIR SourceFile to Rust-specific SourceFile.
pub fn lower_to_rust(
    abstract_ir: &SourceFile,
    config: &RustLowerConfig,
) -> Result<RustCrateOutput, RustLowerError>;

pub struct RustLowerError { pub reason: String }
```

Transformations applied:
- FnDef return types → `Result<_, ExecError>` or `Result<_, Box<dyn Error>>`
- String literals → `.to_string()` conversions where needed
- Struct/Enum definitions → add `#[derive(Debug, Clone)]`
- Add `use` statements from import analysis
- Transport calls → `gunbc_lib_transport::execute_transport()` or `std::fs`
- Generate `Cargo.toml` with appropriate deps

**Tests**:
- [ ] Abstract `fn main()` → Rust `fn main() -> Result<(), Box<dyn Error>>`
- [ ] Abstract `Let("x", Str("hello"))` → Rust `Let("x", Call("to_string", [Str("hello")]))`
- [ ] StructDef → StructDef with `derives: ["Debug", "Clone"]`
- [ ] Generated Cargo.toml is valid TOML with required dependencies
- [ ] `use_exec_runtime: true` adds gunbc-ir/gunbc-exec deps
- [ ] `use_exec_runtime: false` adds only std deps + minimal runtime
- [ ] `cargo test -p daglang-emit` passes, clippy clean

---

### Task 10: AbstractIR → Go Lowering
Lower AbstractIR to Go-specific constructs.

- **Owns**: `core/daglang/daglang-emit/src/lower_go.rs` (new)
- **Reads**: `core/ir/src/code_ir.rs`, `core/ir/src/go_ir.rs` (Task 7)
- **Blocked by**: Task 2, Task 7, Task 8

**Produces**:
```rust
// lower_go.rs

use gunbc_ir::go_ir::GoSourceFile;
use gunbc_ir::code_ir::SourceFile;

pub struct GoModuleOutput {
    pub source_file: GoSourceFile,
    pub go_mod: String,                   // go.mod content
    pub go_sum: Option<String>,
}

pub struct GoLowerConfig {
    pub module_path: String,              // e.g., "github.com/example/makegen"
    pub go_version: String,               // e.g., "1.21"
}

pub fn lower_to_go(
    abstract_ir: &SourceFile,
    config: &GoLowerConfig,
) -> Result<GoModuleOutput, GoLowerError>;

pub struct GoLowerError { pub reason: String }
```

Transformations:
- AbstractIR `fn` → Go `func` with `(result, error)` returns
- AbstractIR `If` → Go `if err != nil { return ..., err }`
- AbstractIR `Let` → Go short declaration `:=`
- AbstractIR string ops → Go `fmt.Sprintf`, `strings.Join`, etc.
- Type mapping via `TypeSystemMapping` for Go
- Import deduction from used packages

**Tests**:
- [ ] Abstract `fn main()` → Go `func main()` with `os.Exit` error handling
- [ ] Abstract `Let("x", Call("concat", ...))` → Go `x := strings.Join(...)`
- [ ] Abstract `If(cond, then, else)` → Go `if cond { ... } else { ... }`
- [ ] Transport call → Go `os.ReadFile` / `os.WriteFile` / `exec.Command`
- [ ] Generated `go.mod` has correct module path and Go version
- [ ] `cargo test -p daglang-emit` passes, clippy clean

---

### Task 11: AbstractIR → C Lowering
Lower AbstractIR to C-specific constructs.

- **Owns**: `core/daglang/daglang-emit/src/lower_c.rs` (new)
- **Reads**: `core/ir/src/code_ir.rs`, `core/ir/src/c_ir.rs` (Task 5)
- **Blocked by**: Task 2, Task 5, Task 8

**Produces**:
```rust
// lower_c.rs

use gunbc_ir::c_ir::CSourceFile;
use gunbc_ir::code_ir::SourceFile;

pub struct CBuildOutput {
    pub source_file: CSourceFile,
    pub header_file: Option<CSourceFile>,   // .h if needed
    pub build_script: String,               // Makefile or build.sh
}

pub struct CLowerConfig {
    pub program_name: String,
    pub use_arena: bool,                    // arena allocator vs manual free
}

pub fn lower_to_c(
    abstract_ir: &SourceFile,
    config: &CLowerConfig,
) -> Result<CBuildOutput, CLowerError>;

pub struct CLowerError { pub reason: String }
```

Transformations:
- AbstractIR `Let` → C `VarDecl` with explicit type
- AbstractIR strings → C `char*` with length tracking
- AbstractIR lists → C `{ pointer, count }` structs
- AbstractIR `fn` → C function with return code + out-params for errors
- Value type → C tagged union `struct Value { enum tag; union { ... } }`
- Memory: arena allocator (bump pointer from sbrk) or explicit malloc/free

**Tests**:
- [ ] Abstract `Let("x", Str("hello"))` → C `const char *x = "hello";`
- [ ] Abstract `Let("items", Array([...]))` → C array + count
- [ ] Abstract `fn main(path: String)` → C `int main(int argc, char **argv)`
- [ ] Transport call → C `fopen/fread/fwrite/fclose` or `popen/pclose`
- [ ] Value type generates correct tagged union struct
- [ ] Generated build script compiles with `gcc -Wall -Werror`
- [ ] `cargo test -p daglang-emit` passes, clippy clean

---

## Wave 4 — Depends on Wave 3

### Task 12: Rust CodeRenderer
Render Rust-specific SourceFile to actual .rs text.

- **Owns**: `core/daglang/daglang-emit/src/render_rust.rs` (new)
- **Reads**: `core/ir/src/code_ir.rs`, `lower_rust.rs` (Task 9)
- **Blocked by**: Task 9

**Produces**:
```rust
// render_rust.rs

/// Render a Rust SourceFile to .rs text content.
pub fn render_rust_source(source: &SourceFile) -> String;

/// Render a full RustCrateOutput to a set of files.
pub fn render_rust_crate(output: &RustCrateOutput) -> Vec<(String, String)>;
// Returns (path, content) pairs: [("src/main.rs", "..."), ("Cargo.toml", "...")]
```

**Tests**:
- [ ] FnDef → `pub fn name(param: Type) -> ReturnType { ... }`
- [ ] EnumDef with derives → `#[derive(Debug, Clone)]\npub enum Name { ... }`
- [ ] ImplBlock → `impl Trait for Type { ... }`
- [ ] Import → `use path::to::{Item1, Item2};`
- [ ] Let → `let name = expr;`, Let(mutable) → `let mut name = expr;`
- [ ] If/else → proper Rust `if cond { ... } else { ... }`
- [ ] Rendered code is valid Rust (syntax check via `rustfmt --check` or similar)
- [ ] `cargo test -p daglang-emit` passes, clippy clean

---

### Task 13: Go CodeRenderer
Render Go IR to actual .go text.

- **Owns**: `core/daglang/daglang-emit/src/render_go.rs` (new)
- **Reads**: `core/ir/src/go_ir.rs` (Task 7), `lower_go.rs` (Task 10)
- **Blocked by**: Task 10

**Produces**:
```rust
// render_go.rs

use gunbc_ir::go_ir::GoSourceFile;

/// Render a GoSourceFile to .go text content.
pub fn render_go_source(source: &GoSourceFile) -> String;

/// Render a full GoModuleOutput to a set of files.
pub fn render_go_module(output: &GoModuleOutput) -> Vec<(String, String)>;
// Returns [("main.go", "..."), ("go.mod", "...")]
```

**Tests**:
- [ ] GoFuncDef → `func Name(param Type) (Result, error) { ... }`
- [ ] GoStmt::ShortDecl → `x, err := someFunc()`
- [ ] GoStmt::If → `if err != nil { return ..., err }`
- [ ] GoStmt::ForRange → `for _, item := range items { ... }`
- [ ] GoImport → `import (\n\t"fmt"\n\t"os"\n)`
- [ ] GoExpr::CompositeLit → `SomeType{field: value}`
- [ ] Package declaration: `package main`
- [ ] `cargo test -p daglang-emit` passes, clippy clean

---

### Task 14: C CodeRenderer
Render C IR to actual .c text.

- **Owns**: `core/daglang/daglang-emit/src/render_c.rs` (new)
- **Reads**: `core/ir/src/c_ir.rs` (Task 5), `lower_c.rs` (Task 11)
- **Blocked by**: Task 11

**Produces**:
```rust
// render_c.rs

use gunbc_ir::c_ir::CSourceFile;

pub fn render_c_source(source: &CSourceFile) -> String;
pub fn render_c_build(output: &CBuildOutput) -> Vec<(String, String)>;
// Returns [("makegen.c", "..."), ("Makefile", "...")]
```

**Tests**:
- [ ] CFunction → `Type name(Type param) { ... }`
- [ ] CStmt::VarDecl → `Type name = init;`
- [ ] CStmt::If → `if (cond) { ... } else { ... }`
- [ ] CStmt::For → `for (init; cond; step) { ... }`
- [ ] CTypeDef::TaggedUnion → `typedef enum { ... } Tag; typedef struct { Tag tag; union { ... }; } Value;`
- [ ] Includes → `#include <stdio.h>`
- [ ] `cargo test -p daglang-emit` passes, clippy clean

---

### Task 15: CStyleIR → MIPS Lowering
Lower C IR to MIPS register-level instructions.

- **Owns**: `core/daglang/daglang-emit/src/lower_mips.rs` (new)
- **Reads**: `core/ir/src/c_ir.rs` (Task 5), `core/ir/src/register_ir.rs` (Task 6)
- **Blocked by**: Task 5, Task 6, Task 11

**Produces**:
```rust
// lower_mips.rs

use gunbc_ir::c_ir::CSourceFile;
use gunbc_ir::register_ir::MipsProgram;

pub fn lower_to_mips(c_ir: &CSourceFile) -> Result<MipsProgram, MipsLowerError>;

pub struct MipsLowerError { pub reason: String }
```

Handles:
- CFunction → labeled block with stack frame prologue/epilogue
- CStmt::VarDecl → stack slot allocation, `sw` to store
- CExpr::Call → argument setup in $a0-$a3 + `jal` + $v0 return
- CStmt::If → conditional branch (`beq`/`bne`) + labels
- String literals → `.data` section entries + `la` references
- File I/O → Linux MIPS syscall sequences (open=4005, read=4003, write=4004, close=4006)
- Simple register allocation: $t0-$t9 for temporaries, $s0-$s7 for across-call values

**Tests**:
- [ ] Empty main → `.text` with `main:` label + `li $v0, 10; syscall` (exit)
- [ ] VarDecl with int → `li $t0, value; sw $t0, offset($sp)`
- [ ] Function call → `move $a0, ...; jal func_label; move $t0, $v0`
- [ ] If/else → `beq $t0, $zero, L_else; ...; j L_end; L_else: ...; L_end:`
- [ ] String literal → `.data` entry + `la $t0, label` in `.text`
- [ ] Stack frame: `addi $sp, $sp, -N` prologue + `addi $sp, $sp, N` epilogue
- [ ] `cargo test -p daglang-emit` passes, clippy clean

---

## Wave 5 — Integration

### Task 16: MIPS CodeRenderer
Render MipsProgram to assembly text.

- **Owns**: `core/daglang/daglang-emit/src/render_mips.rs` (new)
- **Reads**: `core/ir/src/register_ir.rs` (Task 6)
- **Blocked by**: Task 6, Task 15

**Produces**:
```rust
// render_mips.rs

use gunbc_ir::register_ir::MipsProgram;

pub fn render_mips_assembly(program: &MipsProgram) -> String;
```

**Tests**:
- [ ] Data section renders as `.data\nlabel: .asciiz "text"\n`
- [ ] Text section renders as `.text\n.globl main\nmain:\n`
- [ ] Instructions render with correct syntax: `add $t0, $t1, $t2`
- [ ] Labels render as `label_name:\n`
- [ ] Comments render as `# comment text`
- [ ] Full minimal program renders valid MIPS (assemblable with `mips-linux-gnu-as`)
- [ ] `cargo test -p daglang-emit` passes, clippy clean

---

### Task 17: `daglang compile` CLI Command
Wire the full pipeline: parse → lower → derive → plan → lower_ir → lower_lang → render → write.

- **Owns**: `core/daglang/daglang-cli/src/compile.rs` (extend existing), `core/daglang/daglang-cli/src/main.rs` (add subcommand)
- **Reads**: all daglang-emit modules
- **Blocked by**: Task 4 (EmitPlan), at least one renderer (Task 12 or Task 3)

**Produces**:
```
daglang compile <input.dag> --target {rust|go|c|mips} [--out <dir>] [--layer {1|2}]

Exit codes:
  0 = success, files written
  1 = compilation error (parse/type/lower)
  2 = emission error (codegen)
```

**Tests**:
- [ ] `daglang compile dsl/tools/makegen.dag --target rust --out /tmp/test` writes files
- [ ] `daglang compile` with missing `--target` prints usage error
- [ ] `daglang compile nonexistent.dag` exits with code 1
- [ ] `--layer 1` only available for `--target rust`
- [ ] CLI test in `core/daglang/daglang-cli/tests/cli_commands.rs`
- [ ] `cargo test -p daglang-cli` passes, clippy clean

---

### Task 18: Cross-Language Parity Test Harness
Build, run, and compare generated binaries across all targets.

- **Owns**: `core/daglang/daglang-cli/tests/codegen_parity.rs` (new)
- **Reads**: all daglang-emit modules, CLI (Task 17)
- **Blocked by**: Task 17, at least 2 renderers

**Produces**:
```rust
// codegen_parity.rs

// Test that generated binaries produce identical output across targets.
// Each test:
// 1. Compiles .dag to target
// 2. Builds the generated code (cargo build / go build / gcc / mips-as + ld)
// 3. Runs the generated binary with test fixtures
// 4. Captures output file content
// 5. Asserts identical across all targets that compiled
```

**Tests**:
- [ ] `makegen_rust_produces_correct_makefile` — Rust Layer 1 output matches hand-built
- [ ] `makegen_rust_native_produces_correct_makefile` — Rust Layer 2 matches Layer 1
- [ ] `makegen_go_produces_correct_makefile` — Go output matches Rust
- [ ] `makegen_c_produces_correct_makefile` — C output matches Rust
- [ ] `makegen_mips_produces_correct_makefile` — MIPS (via QEMU) output matches Rust
- [ ] `all_targets_produce_identical_output` — cross-target comparison
- [ ] Tests are `#[ignore]` by default (need toolchains), CI enables them

---

## Dependency Graph Summary

```
Wave 1 (parallel, no deps):
  Task 1 (Computation types)
  Task 2 (AbstractIR factor)
  Task 3 (Exec-runtime makegen)

Wave 2 (parallel, need Wave 1):
  Task 4 (EmitPlan)           ← Task 1
  Task 5 (CStyleIR types)     ← Task 2
  Task 6 (RegisterIR types)   ← Task 2
  Task 7 (Go IR types)        ← Task 2

Wave 3 (parallel, need Wave 2):
  Task 8 (→ AbstractIR lower) ← Task 1, 2, 4
  Task 9 (→ Rust lower)       ← Task 2, 8
  Task 10 (→ Go lower)        ← Task 2, 7, 8
  Task 11 (→ C lower)         ← Task 2, 5, 8

Wave 4 (parallel, need Wave 3):
  Task 12 (Rust renderer)     ← Task 9
  Task 13 (Go renderer)       ← Task 10
  Task 14 (C renderer)        ← Task 11
  Task 15 (→ MIPS lower)      ← Task 5, 6, 11

Wave 5 (integration):
  Task 16 (MIPS renderer)     ← Task 6, 15
  Task 17 (CLI compile)       ← Task 4, any renderer
  Task 18 (Parity tests)      ← Task 17, 2+ renderers
```

## File Ownership (No Overlap)

| Task | Creates | Modifies |
|------|---------|----------|
| 1 | `daglang-emit/src/computation.rs` | `daglang-emit/src/lib.rs` (add `pub mod`) |
| 2 | — | `core/ir/src/code_ir.rs` |
| 3 | `daglang-emit/src/rust_exec_runtime.rs` | `daglang-emit/src/lib.rs` (add `pub mod`) |
| 4 | `daglang-emit/src/plan.rs` | `daglang-emit/src/lib.rs` (add `pub mod`) |
| 5 | `core/ir/src/c_ir.rs` | `core/ir/src/lib.rs` (add `pub mod`) |
| 6 | `core/ir/src/register_ir.rs` | `core/ir/src/lib.rs` (add `pub mod`) |
| 7 | `core/ir/src/go_ir.rs` | `core/ir/src/lib.rs` (add `pub mod`) |
| 8 | `daglang-emit/src/lower_to_ir.rs` | `daglang-emit/src/lib.rs` (add `pub mod`) |
| 9 | `daglang-emit/src/lower_rust.rs` | `daglang-emit/src/lib.rs` (add `pub mod`) |
| 10 | `daglang-emit/src/lower_go.rs` | `daglang-emit/src/lib.rs` (add `pub mod`) |
| 11 | `daglang-emit/src/lower_c.rs` | `daglang-emit/src/lib.rs` (add `pub mod`) |
| 12 | `daglang-emit/src/render_rust.rs` | `daglang-emit/src/lib.rs` (add `pub mod`) |
| 13 | `daglang-emit/src/render_go.rs` | `daglang-emit/src/lib.rs` (add `pub mod`) |
| 14 | `daglang-emit/src/render_c.rs` | `daglang-emit/src/lib.rs` (add `pub mod`) |
| 15 | `daglang-emit/src/lower_mips.rs` | `daglang-emit/src/lib.rs` (add `pub mod`) |
| 16 | `daglang-emit/src/render_mips.rs` | `daglang-emit/src/lib.rs` (add `pub mod`) |
| 17 | — | `daglang-cli/src/compile.rs`, `daglang-cli/src/main.rs` |
| 18 | `daglang-cli/tests/codegen_parity.rs` | — |

**Shared file note**: Multiple tasks add `pub mod` lines to `daglang-emit/src/lib.rs`
and `core/ir/src/lib.rs`. To avoid conflicts: each task adds its module declaration
at the end of the module list. Alternatively, a coordinator task (or the first task
in each wave) can pre-add all module declarations as stubs.

**Recommendation**: Before starting Wave 1, a prep task adds all `pub mod` stubs:
```rust
// daglang-emit/src/lib.rs — add at bottom:
pub mod computation;
pub mod plan;
pub mod rust_exec_runtime;
pub mod lower_to_ir;
pub mod lower_rust;
pub mod lower_go;
pub mod lower_c;
pub mod lower_mips;
pub mod render_rust;
pub mod render_go;
pub mod render_c;
pub mod render_mips;

// core/ir/src/lib.rs — add at bottom:
pub mod c_ir;
pub mod go_ir;
pub mod register_ir;
```
Each module starts as `// TODO` and is replaced by the owning task.
