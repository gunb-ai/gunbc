# Multi-Language Code Generation for gunbc

## Goal

Add multi-language code generation to gunbc, similar to the-gunbai's IR-based approach. Currently gunbc generates only Rust code via hardcoded templates. This plan adds a backend abstraction layer to support Python, TypeScript, and other languages.

## Current State

### Already Language-Neutral (keep as-is)
- `core/ir/src/types.rs` - `Cardinality` enum (mathematical foundation)
- `core/codegen/src/registry.rs` - `PortDef`, `NodeDef`, `EdgeDef`, `DagDef` are pure data
- `core/ir/src/language/traits/type_system.rs` - Has `TypeMapping` with Python/TS mappings already defined

### Hardcoded Rust (needs refactoring)
- `core/codegen/src/cli_gen.rs` - Embeds Rust syntax directly (`match`, `eprintln!`, `Vec<String>`, etc.)
- `core/codegen/src/dag_gen.rs` - Generates Rust `graph.rs` with hardcoded syntax

## Architecture

```
PortDef, NodeDef, DagDef (language-neutral)
              │
              ▼
       LanguageBackend trait
              │
    ┌─────────┼─────────┐
    ▼         ▼         ▼
RustBackend  PythonBackend  TypeScriptBackend
    │         │         │
    ▼         ▼         ▼
 main.rs    main.py   main.ts
```

## Implementation Plan

### Phase 1: Backend Trait Foundation

**1.1 Create `core/codegen/src/backend/mod.rs`**

```rust
pub trait LanguageBackend {
    fn language_id(&self) -> &str;
    fn file_extension(&self) -> &str;

    // Type rendering
    fn render_type(&self, type_id: &str, cardinality: &str) -> String;

    // Statement rendering
    fn render_import(&self, module: &str, items: &[&str]) -> String;
    fn render_let(&self, name: &str, type_hint: Option<&str>, value: &str) -> String;
    fn render_print(&self, value: &str) -> String;
    fn render_print_error(&self, message: &str) -> String;
    fn render_exit(&self, code: i32) -> String;

    // CLI-specific
    fn render_arg_parse(&self) -> String;
    fn render_arg_check(&self, arg_name: &str, value_expr: &str) -> String;
    fn render_match_arg(&self, cases: &[(String, String)], default: &str) -> String;

    // Value construction (for the DAG runtime)
    fn render_value_constructor(&self, type_id: &str, expr: &str) -> String;
}
```

**1.2 Create `core/codegen/src/backend/rust.rs`**

Extract current hardcoded patterns from `cli_gen.rs` into `RustBackend`:
- `render_type("String", "One")` → `"String"`
- `render_type("String", "ZeroOrOne")` → `"Option<String>"`
- `render_import("std", &["env", "process"])` → `"use std::{env, process};"`
- `render_let("args", Some("Vec<String>"), "env::args().collect()")` → full let statement
- etc.

### Phase 2: Refactor cli_gen.rs

**2.1 Parameterize `generate_cli_main()`**

Change signature:
```rust
// Before
pub fn generate_cli_main(tool: &ToolDef, dag: &DagDef, ...) -> String

// After
pub fn generate_cli_main<B: LanguageBackend>(
    backend: &B,
    tool: &ToolDef,
    dag: &DagDef,
    ...
) -> String
```

**2.2 Replace hardcoded templates with backend calls**

```rust
// Before (line 203-207)
format!(r#"
use std::env;
use std::process;

fn main() {{
    let args: Vec<String> = env::args().collect();
"#)

// After
format!(r#"
{}
{}

{}
"#,
    backend.render_import("std", &["env", "process"]),
    backend.render_main_signature(),
    backend.render_let("args", Some(&backend.render_type("String", "ZeroOrMore")),
                       &backend.render_arg_parse()),
)
```

### Phase 3: Add Python Backend

**3.1 Create `core/codegen/src/backend/python.rs`**

```rust
impl LanguageBackend for PythonBackend {
    fn language_id(&self) -> &str { "python" }
    fn file_extension(&self) -> &str { "py" }

    fn render_type(&self, type_id: &str, cardinality: &str) -> String {
        let base = match type_id {
            "String" => "str",
            "Int" => "int",
            "Bool" => "bool",
            _ => type_id,
        };
        match cardinality {
            "ZeroOrOne" => format!("{} | None", base),
            "ZeroOrMore" | "OneOrMore" => format!("list[{}]", base),
            _ => base.to_string(),
        }
    }

    fn render_import(&self, module: &str, items: &[&str]) -> String {
        format!("from {} import {}", module, items.join(", "))
    }

    fn render_let(&self, name: &str, type_hint: Option<&str>, value: &str) -> String {
        match type_hint {
            Some(t) => format!("{}: {} = {}", name, t, value),
            None => format!("{} = {}", name, value),
        }
    }

    fn render_print(&self, value: &str) -> String {
        format!("print({})", value)
    }

    fn render_print_error(&self, message: &str) -> String {
        format!("print({}, file=sys.stderr)", message)
    }

    fn render_exit(&self, code: i32) -> String {
        format!("sys.exit({})", code)
    }

    fn render_arg_parse(&self) -> String {
        "sys.argv".to_string()
    }
    // ... etc
}
```

### Phase 4: Add TypeScript Backend

Similar to Phase 3, implementing TypeScript-specific syntax.

### Phase 5: Refactor dag_gen.rs

Apply same pattern to DAG generation - this is lower priority since DAGs are primarily a Rust runtime concept currently.

### Phase 6: Integration

**6.1 Add `--language` flag to gunbc-codegen**

```rust
#[derive(clap::ValueEnum, Clone)]
pub enum TargetLanguage {
    Rust,
    Python,
    TypeScript,
}

// In codegen CLI
#[arg(long, default_value = "rust")]
language: TargetLanguage,
```

**6.2 Update output paths**

- Rust: `buck-out/gen/bin/{tool}/main.rs`
- Python: `buck-out/gen/bin/{tool}/main.py`
- TypeScript: `buck-out/gen/bin/{tool}/main.ts`

## Files to Modify

| File | Change |
|------|--------|
| `core/codegen/src/backend/mod.rs` | NEW - Backend trait |
| `core/codegen/src/backend/rust.rs` | NEW - Rust implementation |
| `core/codegen/src/backend/python.rs` | NEW - Python implementation |
| `core/codegen/src/backend/typescript.rs` | NEW - TypeScript implementation |
| `core/codegen/src/cli_gen.rs` | Refactor to use backend trait |
| `core/codegen/src/dag_gen.rs` | Refactor to use backend trait (Phase 5) |
| `core/codegen/src/lib.rs` | Export new backend module |
| `lib/tools/codegen/src/main.rs` | Add `--language` flag |

## Testing Strategy

1. **Snapshot tests**: Generate CLI for each language, compare to golden files
2. **Round-trip tests**: Generate Python, run it, verify same output as Rust version
3. **Type mapping tests**: Unit tests for each backend's `render_type()`

## Dependencies

- Python backend: Requires Python runtime for execution
- TypeScript backend: Requires `ts-node` or compilation step

## Open Questions

1. Should DAG execution work in Python/TS, or just CLI generation?
   - If execution needed: Port `gunbc-exec` concepts to each language
   - If CLI-only: Generate standalone scripts that shell out to Rust executor

2. How to handle Rust-specific ops (like `PrimitiveOp::Parse`)?
   - Option A: Generate equivalent Python/TS code inline
   - Option B: Generate FFI calls to Rust library
   - Option C: Reimplement primitive ops in each language

## Reference

See the-gunbai's implementation:
- `crates/gunbai-integrations-contracts/src/codegen/backend.rs`
- `crates/gunbai-integrations-contracts/src/codegen/ir.rs`
- `crates/gunbai-integrations-contracts/src/understanding/languages/`
