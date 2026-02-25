# DAG Modeling Guide

How to model things in `.dag` files so the compiler can enforce them across the entire repo.

**Audience**: Anyone adding or modifying `.dag` files.
**When to read this**: Before writing a new `.dag` file, or when an extern bridge feels wrong.

---

## The core idea

A `.dag` file declares **what is true**. Rust compiles and executes — it doesn't decide
what's true. When modeling works, you write a data declaration once and the compiler
derives clippy configs, Makefile targets, `.gitignore` entries, CLI flags, and CI gates
from it. When modeling fails, you end up maintaining the same fact in three places.

---

## File layout and layers

```
dsl/
├── std/           Layer 0 — Universal types and pure functions
├── extdeps/       Layer 1 — Facts about external systems (no opinions)
├── config/        Layer 2 — Our repo's rules (composing Layer 0 + 1)
└── tools/         Layer 3 — Executable workflows (consuming all layers)
```

Each layer has a specific job:

| Layer | Contains | Example | Rule |
|-------|----------|---------|------|
| `std/` | Types, pure data, pure fns | `NamingCase`, `MarkdownNode`, `BoxChars` | No policy. No tool-specific logic. |
| `extdeps/` | Domain models of external things | "What is clippy?", "What is GNU Make?" | Facts only. Zero opinions about our repo. |
| `config/` | Our invariants, exemptions, policies | I6 (no escape hatches), warning policy | Composes extdeps + std. No executable logic yet. |
| `tools/` | Funcs that run and produce artifacts | `makegen`, `pragma`, `testgen` | Consumes all layers. Owns `@outputs`. |

**Import direction**: `tools/ → config/ → extdeps/ → std/`. Never backwards.

---

## Pattern 1: Tautological data declarations

A tautology is a statement that's true by definition. Model external systems as
tautologies — facts that can't be wrong because they just describe what the thing *is*.

### Example: "What is Rust?" (`std/languages.dag`)

```
data rust_language: Language = {
  id: "rust",
  comment: { line_prefix: "//" },
  naming: { type_case: PascalCase, function_case: SnakeCase },
  types: { string: "String", int: "i64", bool: "bool" }
}
```

This answers a universal question. Every part of the codebase that needs to know how
Rust names things imports this one declaration. No second copy.

### Example: "What is clippy?" (`extdeps/clippy.dag`)

```
type ClippySpec { id: String, config_file: String, integration: String }
type LintLevel = Deny | Warn | Allow
type ClippyCategory { name: String, default_level: LintLevel }

data spec: ClippySpec = { id: "clippy", config_file: "clippy.toml", integration: "cargo clippy" }

data categories: List<ClippyCategory> = [
  { name: "correctness", default_level: Deny },
  { name: "suspicious",  default_level: Warn },
  { name: "style",       default_level: Allow },
  // ...
]
```

This is Layer 1 — facts about clippy's surface area. No opinion about what *we* deny.
That's Layer 2's job.

### Example: "What is GNU Make?" (`extdeps/make.dag`)

```
type MakeTarget { name: String, deps: List<String>, commands: List<String>, phony: Bool }
type MakeVariable { name: String, value: String }
type Makefile { variables: List<MakeVariable>, sections: List<MakeSection> }
```

Domain vocabulary. Tools that generate Makefiles import these types.

### How it flows to enforcement

Once `extdeps/clippy.dag` declares clippy's categories, `config/clippy_policy.dag`
composes them with our invariants to derive the actual `clippy.toml`. The tool generates
the file. CI tests verify the generated file matches the model. One source of truth,
three enforcement points.

---

## Pattern 2: Invariant layering with scoped exemptions

Invariants are repo rules. Model them as tiered data — the invariant itself, the boundary
rules it creates, and the specific exemptions.

### Example: Architectural rules (`config/arch_rules.dag`)

**Tier 1 — The invariant:**
```
type Invariant { id: String, name: String, description: String }

data i6: Invariant = {
  id: "I6",
  name: "No Escape Hatches",
  description: "Direct filesystem/process/network I/O is disallowed outside transport boundaries."
}
```

**Tier 2 — What it restricts (resources, not libraries):**
```
type BoundaryRule { invariant_id: String, restricted_resource: String, rationale_template: String }

data boundary_rules: List<BoundaryRule> = [
  { invariant_id: "I6", restricted_resource: "Filesystem",
    rationale_template: "Direct {resource} access bypasses transport boundary audit trail." },
  { invariant_id: "I6", restricted_resource: "Process", ... },
  { invariant_id: "M7", restricted_resource: "SecretTransport", ... },
]
```

**Tier 3 — Exemptions tied to *specific* invariants:**
```
data exemptions: List<Exemption> = [
  // This package IS the I/O boundary — it's exempt from I6 by definition
  { invariant_id: "I6", scope: WholePackage { name: "gunbc-lib-transport" },
    rationale: "This package IS the transport boundary." },

  // Compiler needs filesystem for source discovery — scoped exemption
  { invariant_id: "I6", scope: PackagePrefix { prefix: "daglang-" },
    rationale: "Compiler filesystem discovery for source files." },
]
```

**Why tiered**: An I6 exemption (filesystem) does NOT grant an M7 exemption (secrets).
Flat allowlists can't express this. Tiered invariants can.

### How it flows to enforcement

1. `config/clippy_policy.dag` imports `boundary_rules` and `exemptions`
2. Derives which crates get relaxed `clippy.toml` (disallowed methods per crate)
3. Derives which files are allowed `#[allow(dead_code)]`
4. `tools/pragma` generates `clippy.toml`, allowlist, lint policy files
5. CI runs `cargo clippy -- -D warnings` — the generated configs enforce the invariants
6. Drift tests verify generated files match the model

One data change in `arch_rules.dag` propagates to clippy config, pragma policy, and CI.

---

## Pattern 3: Policy composition via imports

Layer 2 (config) composes Layer 0 (std) + Layer 1 (extdeps) to derive policy.

### Example: Clippy policy (`config/clippy_policy.dag`)

```
import extdeps.clippy { ClippySpec, DisallowedMethodEntry, ClippyTomlConfig, spec }
import config.arch_rules { BoundaryRule, Exemption, boundary_rules, exemptions,
                           dead_code_allowances, pragma_allow_lints, large_error_threshold }

// Compose: clippy surface + our invariants → derived artifacts
func derive_clippy_toml() -> String { ... }
func derive_disallowed_methods_allowlist() -> String { ... }
func derive_pragma_lint_policy() -> String { ... }
```

The functions consume both layers. Today they're backed by extern impls (Phase 1).
When the lowerer supports multi-source filter/map, they become pure DSL (Phase 2).
Either way, the *modeling* is right — the data flows from declarations to derived outputs.

### Example: Build targets (`config/build_targets.dag`)

```
import extdeps.make { CoreWorkflow, MetaTargetDef }

data core_workflows: List<CoreWorkflow> = [
  { name: "preflight-fix",   description: "Auto-fix formatting + lint", deps: [] },
  { name: "ensure-codegen",  description: "Run codegen if stale",      deps: ["preflight-fix"] },
  { name: "build",           description: "Compile workspace",         deps: ["ensure-codegen"] },
  { name: "verify",          description: "Full verification",         deps: ["build", "test-all", "lint-upsert"] },
  // ...20 total
]

data meta_targets: List<MetaTargetDef> = [
  { name: "test",  description: "Run all tests",  has_check: true,  has_fix: false },
  { name: "clippy", description: "Run clippy",     has_check: true,  has_fix: true },
  // ...7 total
]
```

These declarations are the **single source of truth** for Makefile generation. The
`makegen` tool reads them, renders targets with correct dependency ordering, and emits
a Makefile. CI verifies the Makefile matches.

---

## Pattern 4: Workspace as external dependency

Your own repo is an external system from the compiler's perspective. Model it that way.

### Example: Workspace model (`extdeps/gunbc.dag`)

```
type CrateRole = TransportBoundary | Compiler | CodegenBootstrap | Runtime | Infra | Library | Tool

type WorkspacePackage { name: String, role: CrateRole, description: String, io_boundary: Bool }

data packages: List<WorkspacePackage> = [
  { name: "gunbc-lib-transport", role: TransportBoundary,
    description: "I/O boundary for all external operations", io_boundary: true },
  { name: "daglang-syntax",     role: Compiler,
    description: "DSL lexer and parser", io_boundary: false },
  // ...15 total
]
```

This lets `config/arch_rules.dag` reference workspace roles when defining exemptions,
and lets `tools/makegen.dag` generate per-crate targets from structure instead of
hard-coded lists.

---

## Pattern 5: The full enforcement chain

Here's how a single data declaration becomes a repo-wide invariant, using Make as
the concrete example.

**Step 1 — Declare the data** (`config/build_targets.dag`):
```
data core_workflows: List<CoreWorkflow> = [
  { name: "verify", deps: ["build", "test-all", "lint-upsert"] },
]
```

**Step 2 — Tool consumes the data** (`tools/makegen.dag`):
```
@outputs("Makefile")
func makegen(path: String = "Makefile") -> { written: Bool } {
  content = render_makefile_content(path: path)
  result = content_upsert(content: content, path: path)
  return { written: result.written }
}
```

**Step 3 — Registry discovers the tool** (structural entrypoint inference):
The compiler sees `makegen` is a zero-arg func, infers it as an entrypoint, extracts
`@outputs("Makefile")`, and registers it in the tool catalog.

**Step 4 — Codegen generates CLI + Makefile** (`makegen/registry.rs` → `makegen/render.rs`):
The registry populates `ToolRegistry` from DSL discovery. The renderer iterates tools
and workflows, emitting structured Makefile blocks. Each tool gets a target with correct
dependencies, help text, and Make variables derived from DSL func params.

**Step 5 — Gitignore derived from outputs** (`makegen/gitignore.rs`):
Tool outputs (from `@outputs` annotations) become `.gitignore` entries, minus
bootstrap seed files declared in `config/arch_rules.dag`.

**Step 6 — CI drift tests verify alignment** (`tests/tool_registration.rs`):

| Test | What it checks |
|------|----------------|
| `tool_declared_outputs_match_dsl_compilation` | `CompileOutput.output_paths` == `ToolDef.outputs` |
| `no_generated_files_committed` | `git ls-files` finds no tracked outputs (except seeds) |
| `all_tool_outputs_gitignored` | `git check-ignore` passes for every output path |
| `dsl_warning_policy_matches_build_config` | DSL `DenyAll` == Rust `Warnings::Deny` |
| `makegen_default_registry_matches_dsl_tools` | Makefile tool set == DSL tool set |
| `workspace_binary_enum_covers_dsl_tools` | `WorkspaceBinary` enum has a variant per tool |
| `dsl_is_single_authority` | No CLI tools exist outside DSL discovery |

**Result**: Change one workflow in `build_targets.dag` → Makefile updates → gitignore
updates → CI enforces. No manual coordination.

---

## Pattern 6: `@outputs` for artifact traceability

Every tool that writes files declares what it writes:

```
@outputs("Makefile")
func makegen(...) -> { ... }

@outputs("**/generated_tests*.rs")
func testgen(...) -> { ... }

@outputs("target/codegen/.stamp")
func codegen(...) -> { ... }
```

The compiler extracts these annotations (plus `content_upsert` literal paths) into
`CompileOutput.output_paths`. This feeds:
- `.gitignore` generation (all outputs are ignored)
- Commit policy (generated files must not be tracked, except seeds)
- Clean/rollback (tools know what they produced)

**Seed files** (`.gitignore`, `clippy.toml`, `deps.toml`) are generated *and* committed
as bootstrap prerequisites. They're declared in `config/arch_rules.dag`:

```
data bootstrap_seed_files: List<String> = [
  ".gitignore", "clippy.toml", "deps.toml", "docs/ab-writing-workflows.md"
]
```

Drift tests specifically exclude seeds from the "no generated files committed" check.

---

## Anti-patterns (lessons learned the hard way)

### 1. Anemic modeling: returning `String` when structural types exist

**Wrong:**
```rust
// extern impl: builds markdown via format!()
fn build_snapshot_content(branch: &str, files: &[String]) -> String {
    format!("# Snapshot\n\nBranch: `{}`\n\n{}", branch, render_tree(files))
}
```

**Right:** Return `MarkdownDoc`, let a renderer stringify it.
```
// DSL or bridge fn returns structure
MarkdownDoc { nodes: [
  Heading { level: 1, text: "Snapshot" },
  Paragraph { text: "Branch: `{branch}`" },
  Tree { paths: sorted_files },
]}
```

The DSL already has `MarkdownNode`, `MarkdownDoc`, `BoxChars`. When bridge fns collapse
to `String` early, those types become dead weight and you can't re-render to different
formats or compose documents.

**Test**: If your extern func returns `String` and the `std/` layer has a structural
type for that domain, the modeling is anemic.

### 2. Scattered data ownership: same fact in Rust and DSL

**Wrong:**
```rust
// In pragma.rs — hardcoded const array
const PRAGMA_ALLOW_LINTS: &[&str] = &[
    "clippy::large_enum_variant",
    "clippy::too_many_arguments",
    "clippy::vec_init_then_push",
    "rustc::unused_variables",
];
```
```
// In config/arch_rules.dag — same data
data pragma_allow_lints: List<String> = [
    "clippy::large_enum_variant",
    "clippy::too_many_arguments",
    "clippy::vec_init_then_push",
    "rustc::unused_variables",
]
```

Two copies. One will drift. The DSL declaration should be the single source; Rust
should consume it at compile time (Phase 2) or be covered by a drift test (Phase 1).

**Test**: Search for `const .*: &\[` and `default_.*() -> Vec` in Rust. If the same
data exists in a `.dag` file, you have scattered ownership.

### 3. Policy in a test, not in the model

**Wrong:**
```rust
#[test]
fn expose_plaintext_callsites_are_in_approved_modules() {
    let approved = &[
        "lib/transport/src/ops.rs",
        "lib/transport/src/executor.rs",
        // ...10 hardcoded paths
    ];
    // scan for .expose_plaintext_for_transport() and fail if not in list
}
```

**Right:** Derive the approved list from the model. `config/arch_rules.dag` already
has M7 (transport boundary secrets) and `extdeps/gunbc.dag` knows which packages are
transport boundaries. The test should verify code matches the model, not maintain
its own parallel list.

### 4. Importing fn items from std modules

**Wrong:**
```
// std/markdown.dag — types AND functions
type MarkdownNode = Heading { ... } | CodeBlock { ... }
fn render_heading(level: Int, text: String) -> String { ... }
```

The lowerer creates DAG nodes for ALL fn items in imported modules, even transitively.
Importing this module injects unconnected `render_heading` nodes that fail with
"missing input" at runtime.

**Right:** Separate types from functions:
```
// std/markdown.dag — types ONLY
type MarkdownNode = Heading { ... } | CodeBlock { ... }

// std/markdown_render.dag — functions (separate module)
import std.markdown { MarkdownNode }
fn render_heading(level: Int, text: String) -> String { ... }
```

### 5. Same-module extern func calls

**Wrong:**
```
// tools/foo.dag
extern func heavy_computation(x: String) -> String

func do_work() -> String {
  result = heavy_computation(x: "input")  // BREAKS: same-module extern call
  return result
}
```

The lowerer doesn't wire `ExternCall` output ports correctly for same-module calls.

**Right:** Use cross-module imports, or keep a shadow fn body:
```
// tools/foo.dag
fn heavy_computation(x: String) -> String {
  ""  // placeholder body; overridden by extern impl at resolve time
}

func do_work() -> String {
  result = heavy_computation(x: "input")  // works: shadow fn, not extern
  return result
}
```

This is a known lowerer limitation (NF-7), not a design choice.

### 6. String IDs where enums would do

**Wrong:**
```rust
type PackageManagerId = String;  // "apt", "brew", "cargo" — open-ended
```

**Right:**
```
type PackageManager = Apt | Brew | Cargo | Npm
```

Closed enums make exhaustive matching possible. The compiler catches missing cases.
String IDs push validation to runtime.

### 7. Rendering to String too early in the pipeline

**Wrong:**
```rust
fn generate_clippy_toml(config: &ClippyConfig) -> String {
    let mut out = String::new();
    out.push_str("# Generated by pragma tool\n");
    for method in &config.disallowed_methods {
        out.push_str(&format!("  {{ path = \"{}\", reason = \"{}\" }},\n", method.path, method.reason));
    }
    out
}
```

**Right:** Build structured IR first, render to string at the boundary:
```
ClippyTomlConfig {
  disallowed_methods: [ { path: "std::fs::read", reason: "I6: ..." } ],
  disallowed_types: [ ... ],
}
// Then: render(config) → String  (single render step, at the end)
```

Structured IR enables validation, diffing, multi-format output, and testing without
string parsing.

---

## How to decide where something goes

When you need to model something new:

| Question | Answer | Layer |
|----------|--------|-------|
| Is this a universal truth about an external system? | "Clippy has 9 lint categories" | `extdeps/` |
| Is this our repo's rule or policy? | "We deny all clippy warnings" | `config/` |
| Is this a reusable type or algorithm? | `MarkdownNode`, `BoxChars` | `std/` |
| Does it produce a file or run a command? | Generates `Makefile` | `tools/` |
| Is this a fact about our own workspace? | "gunbc-ir is an Infra crate" | `extdeps/gunbc.dag` |

**If it's data that exists as a Rust const array today**: It probably belongs in
`config/` or `extdeps/` as a data declaration, with the Rust code consuming the
compiled output instead of owning the truth.

**If it's a renderer**: It should accept structured types and produce strings only at
the final step. If you're calling `format!()` or `push_str()` deep in the pipeline,
the modeling is too thin.

---

## The one-representation principle

Every fact should have exactly one canonical source. The ideal state:

| Concern | Canonical source | Rust's role |
|---------|-----------------|-------------|
| "What is clippy?" | `extdeps/clippy.dag` | Compile + execute |
| "What do we deny?" | `config/arch_rules.dag` | Compile + execute |
| "What does clippy.toml contain?" | `config/clippy_policy.dag` | Compile + execute |
| "How is clippy.toml rendered?" | `tools/pragma.dag` | Execute transport |
| Workspace crate roles | `extdeps/gunbc.dag` | Compile + execute |
| Build workflow ordering | `config/build_targets.dag` | Compile + execute |
| Makefile content | `tools/makegen.dag` | Execute transport |

Rust compiles and executes. It doesn't decide what lints to deny, which crates are
exempt, or how workflows are ordered. That's the model's job.

---

## Current state and known gaps

**Working well** (use these as templates for new modeling):
- `extdeps/clippy.dag` + `config/clippy_policy.dag` — full composition chain
- `config/arch_rules.dag` — tiered invariants with scoped exemptions
- `config/build_targets.dag` → `tools/makegen.dag` → Makefile generation
- `std/languages.dag` — tautological data declarations
- `std/box_draw.dag` — pure DSL algorithms (fold, pattern matching)

**Phase 1 scaffolding** (extern impls back the functions, awaiting Phase 2 pure DSL):
- `clippy_policy.dag` derive functions — need multi-source filter/map in lowerer
- `tools/makegen.dag` render function — loads registry from Rust
- `tools/pragma.dag` render functions — reads Rust const arrays

**Known lowerer limitations** (workarounds documented above):
- Fn items in imported modules create unconnected nodes → separate types from fns
- Same-module extern func calls break data flow → use shadow fn bodies
- `@outputs` annotation after `extern asset` misparses → add intervening item

**Migration targets** (Rust const arrays → DSL data declarations):
- `DISALLOWED_METHODS_ALLOWLIST_RULES` → `config/` data
- `DEAD_CODE_ALLOW_RULES` → `config/arch_rules.dag` (already partially there)
- `default_core_workflows()` → `config/build_targets.dag` (already there, Rust not yet consuming)
- `expose_plaintext` approved modules → derive from `extdeps/gunbc.dag` + `arch_rules`

---

## References

- `docs/handbook.md` — Compositional modeling philosophy, layered concerns
- `docs/design/v4/extern-bridge-gap-analysis.md` — Why externs exist, elimination plan
- `docs/design/modeling/repo-self-understanding.md` — Workspace as external system
- `core/infra/src/workspace_model.rs` — Generator graph, tier enforcement
- `gunbc-dag/tests/tool_registration.rs` — Drift detection test suite
