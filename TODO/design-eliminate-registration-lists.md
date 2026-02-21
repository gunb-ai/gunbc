# Eliminate Registration Lists: Close the DSL-Runtime Gap

**Status**: PROPOSED
**Date**: 2026-02-21
**Track**: Cleanup — eliminate hardcoded metadata duplication
**Prerequisite**: CL1-CL8 completed (hardcoded lists consolidated)

## Vision

The DSL is the only programming language for tool, workflow, and pipeline logic.
Rust is infrastructure — compiler, executor, transport adapters — not a fallback
for "complex" logic. There is no escape hatch. If something can't be written in
DSL today, that's a missing DSL feature to be fixed, not a reason to write Rust.

## Problem Statement

The Rust runtime maintains handwritten registries that duplicate metadata the DSL
compiler already knows. Today (post-CL1-CL8), adding a new DSL module still
requires touching Rust code in up to 4 places. Worse, 5 modules implement their
function bodies in Rust — pure computations (string rendering, list filtering,
JSON construction) that belong in DSL but leak into Rust because the DSL lacks
expression-level primitives.

**Goal**: Make it so that adding or modifying any tool, callable, workflow, or
configuration requires **only DSL changes**. Zero Rust edits. Drift is
structurally impossible. Rust as escape hatch is eliminated.

## Why can't we "just write DSL" for all of this?

Short answer: **we almost can.** The remaining Rust exists for three reasons,
none of which are fundamental.

### What's already DSL-only

**Every tool graph is 100% DSL-compiled at runtime.** There are zero hand-coded
Rust DAGs. When `build_pragma_graph()` runs, it calls:

```
dsl/tools/pragma.dag -> daglang_driver::compile -> Dag<LoweredOp> -> resolve -> Dag<DynOp>
```

All graph structure, wiring, and orchestration comes from the DSL. The `emit`
phase can even generate complete Rust/Go/C source code from the compiled DAG.
The DSL already expresses:
- Module dependencies and imports
- Function signatures (inputs, outputs, types)
- Graph topology (data flow, parallelism, stages)
- Resource annotations (`@file(READ/WRITE)`, `@hermetic`, `@mock_response`)
- Service protocol specs (REST endpoints, shell commands, field mappings)
- Pipeline stage ordering

### The three reasons Rust code still exists

#### Reason 1: Leaf-node function bodies (missing DSL feature)

The DSL declares functions like `fn render_clippy_toml(directives) -> String`
but the **body** is implemented in Rust (`PragmaOp::RenderClippy`). These are
pure computations: string templating, list filtering, JSON serialization. The
DSL has the type system for this, but no expression language for function bodies.

This is a **missing DSL feature**, not an architectural limitation. Evidence:
- `InfraToolOp` duplicates logic already expressed in `dsl/tools/infra.dag`
  (match/filter/count) — the DSL CAN express it, but Rust reimplements it
- `PragmaOp` renders strings from config — expressible with string interpolation
- `MakegenOp::LoadRegistry` serializes a Rust struct — needs a DSL-side data
  source or FFI mechanism

**Fix**: Add expression-level DSL support (string ops, list ops, arithmetic).
See "DSL Language Features Required" below for the complete inventory.

#### Reason 2: The resolver (unnecessary — compiler already has the information)

`resolve_lowered_dag()` maps `LoweredOp` (compiler output) to `DynOp`
(executable). The `LoweredOp` already carries module, name, obligation category,
and service metadata — everything needed to route. But the resolver maintains
its own copy of this routing table:

| Registry | What it duplicates |
|---|---|
| `PASSTHROUGH_CALLABLES` (30+ entries) | "These callables exist and are passthrough" — compiler already validated this |
| `resolve_domain()` match arms (6 modules) | "These modules have custom ops" — could be inventory-discovered |
| `resolve_std_resources()` name match | "These resources exist" — compiler knows from `std/resources.dag` |

This is **entirely eliminable** without DSL changes. The compiler proves
callables exist; the resolver should trust that proof. See Changes 1-2 below.

#### Reason 3: Workflow specs are Rust-constructed DAGs (should be DSL pipelines)

The workflow builders (`gist_workflow_spec`, `bootstrap_workflow_spec`, etc.)
construct `Dag<WorkflowUnit>` objects in Rust using `dag.add_node()`/
`dag.add_edge()`. This is the same thing the DSL does — defining graph topology
— but bypassing the compiler entirely.

Meanwhile, `pipelines/ci.dag` and `pipelines/sdlc.dag` already express
pipelines in DSL that the compiler handles. The 12 remaining Rust-constructed
workflows exist because they were written before the DSL pipeline feature was
mature enough.

This is a **migration gap**, not a limitation. The DSL's `pipeline` construct
can express everything the Rust builders do. Evidence: `pipelines/ci.dag` is
the most complex workflow and it's fully DSL.

**Fix**: Migrate workflow builders to `dsl/pipelines/*.dag` files. The process
unit claims (currently in `process_registry.rs`) can be derived from the DSL's
`@file(READ/WRITE)` annotations, which already exist but aren't extracted.

### Summary: What blocks "just write DSL"

| Blocker | Category | How many registries it causes | Fix |
|---|---|---|---|
| Resolver doesn't trust compiler | Architecture gap | 3 (PASSTHROUGH_CALLABLES, match arms, resource names) | Default-passthrough + inventory (this design) |
| Workflow specs in Rust | Migration gap | 2 (TOOL_WORKFLOWS, process_registry) | Migrate to DSL pipeline definitions |
| No function body expressions | Missing DSL feature | 1 (custom Executable impls) | DSL expression language |

None of these are fundamental. All three are fixable.

## DSL Language Features Required

Auditing every custom `Executable` impl (27 op variants across 5 modules)
reveals the exact language primitives the DSL needs to eliminate Rust as an
escape hatch. Every computation in these modules is pure — no I/O, no FFI,
no unsafe — just data transformation between transport boundaries.

**Key insight**: most "string manipulation" in these modules is actually a
**structured modeling failure**. Of ~60 string operations across all modules:
- ~25% are **deserialization** — parsing raw strings into structured data that
  should have arrived structured in the first place
- ~12% are **normalization** — cleaning up data that should arrive clean
- ~8% are **path manipulation** — building paths from string parts instead
  of using structured path types

These categories should be **eliminated by better modeling**, not supported
with string methods. The remaining ~55% is legitimate serialization (rendering
structured data to text output) which needs proper support.

### Principle: Eliminate string manipulation through structured modeling

Before adding string primitives to the DSL, we should ask: **why is this data
a string at all?** In most cases, the answer is "because the transport layer
or data model didn't provide structure."

#### Problem 1: Transport responses arrive as raw strings

Bootstrap parses shell output with 5 chained string operations:
```
shell.stdout.lines()        // split raw text
  .map(|l| l.trim())        // clean whitespace
  .filter(|l| !l.is_empty())  // drop blanks
  .filter_map(|l| l.strip_prefix("crates/"))  // extract path component
  .filter(|n| !n.contains('/'))  // validate single segment
```

This entire chain exists because `find` returns raw text. The DSL shouldn't
need string methods for this — the **transport layer should return structured
records**. Similarly, codegen parses glob responses as newline-separated
strings when they should arrive as `List[FilePath]`.

**Fix**: Structured transport responses. When a shell command's output format
is known (declared in the DSL's service spec), the transport adapter parses it
into typed records before the DAG node ever sees it. The DSL declares the
expected structure; the runtime delivers it.

```
// Instead of returning raw stdout:
service workspace.shell {
  fn find_crates() -> List[CrateName] {
    @shell("find crates -maxdepth 1 -mindepth 1 -type d")
    @parse(lines, strip_prefix: "crates/", filter: single_segment)
  }
}
```

This eliminates `.lines()`, `.trim()`, `.strip_prefix()`, `.contains('/')`,
`.is_empty()` — not by supporting them in DSL, but by making them unnecessary.

#### Problem 2: Paths are strings instead of structured types

Pragma and codegen do path manipulation via string operations:
```
path.to_string_lossy().replace('\\', "/")   // normalization
!normalized.ends_with('/')                   // validation
format!("{}/**/main.rs", codegen_bin_dir())  // pattern building
format!("{}/{}/main.rs", bin_dir, tool_name) // path construction
```

None of this should exist. The DSL already has `FilePath` as a type — it
should be a structured type with segments, not a string alias.

**Fix**: Structured path and glob types in the DSL type system:

```
// Path construction is structural, not string interpolation:
let path = codegen_bin_dir / tool_name / "main.rs"

// Glob patterns are a type, not a formatted string:
let pattern = glob(codegen_bin_dir, "**", "main.rs")
```

The compiler guarantees path separator handling, normalization, and
validation. No string operations needed.

#### Problem 3: Type predicates encoded as string checks

Pragma filters crates by name prefix using `crate_name.starts_with(prefix)`.
This is a type predicate disguised as a string operation — the DSL should
express crate selection as structured matching:

```
// Instead of: crate_name.starts_with("gunbc-lib-")
// Use structured selector:
match crate {
  Crate(prefix: "gunbc-lib-") => apply_policy(...)
  _ => default_policy(...)
}
```

#### What this eliminates from the "string methods" inventory

| Original "string feature" | Eliminated by | Remaining need |
|---|---|---|
| `.lines()`, `.trim()`, `.split()` | Structured transport responses | None |
| `.strip_prefix()`, `.strip_suffix()` | Structured transport parsing | None |
| `.replace('\\', "/")` | Structured path types | None |
| `.contains('/')`, `.ends_with('/')` | Structured path types | None |
| `.starts_with(prefix)` | Structured pattern matching | None |
| `.is_empty()` | Option types / empty-collection handling | Minimal |
| `.len()` | Collection `.len()` (not string-specific) | As list op |

**After proper modeling, no general-purpose string method library is needed.**

### Feature 1: Structured rendering (text output from typed data)

The ~40% of string operations that ARE legitimate — serialization — follow a
consistent pattern: rendering structured data into a text file format (TOML,
Makefile, gitignore, status reports). This is the one place where the DSL
genuinely needs text composition support.

But even here, it shouldn't be ad-hoc string concatenation. It should be
**structured document rendering** — a DAG of typed blocks that compose into
the final output:

```
// Pragma renders a TOML-like config file:
render clippy_toml(policy: ClippyPolicy) -> TextFile {
  section header {
    comment "Generated by gunbc-pragma"
    comment "Do not edit manually"
    blank
  }
  section disallowed_methods {
    for rule in policy.allowlist_rules {
      comment rule.rationale
      line rule.pattern
    }
  }
  section allow_dead_code {
    if policy.dead_code_paths.is_empty() {
      comment "(none)"
    } else {
      for path in policy.dead_code_paths {
        line path
      }
    }
  }
}
```

This is fundamentally a **document DAG** — sections contain blocks, blocks
contain lines, lines contain values. The DSL already models DAGs. The
rendering engine handles:
- Line breaks between sections
- Comment prefixes (`#`, `//`)
- Indentation levels
- Empty-section placeholders

**Required primitives**:
- `render` functions that produce `TextFile` / `Document` types
- `section`, `line`, `comment`, `blank` block constructors
- `for ... in` iteration within render blocks
- `if/else` conditional sections
- `"${expr}"` interpolation within line values

### Feature 2: Collection operations

List/set operations are genuinely needed and used across all modules. These
are not string operations — they operate on typed collections.

**Required primitives**:
- `.map(fn)`, `.filter(fn)` — transform/select
- `.sort()`, `.dedup()` — ordering
- `.any(fn)`, `.all(fn)` — predicate testing
- `.len()` — count
- `.contains(item)` — membership
- `.join(sep)` — render list as delimited string (rendering only)
- List literals `[a, b, c]`

### Feature 3: Pattern matching and conditionals

Used by all 5 modules for dispatch, validation, and branching.

**Required primitives**:
- `match expr { pattern => body, ... }` — exhaustive matching
- `if cond { a } else { b }` — conditional expressions
- `let ... = ...` — binding with destructuring
- Boolean operators: `&&`, `||`, `!`

### Feature 4: Integer arithmetic and comparison

Used by CodegenOp (manifest freshness), BuildOp (exit code checking),
MakegenOp (counting).

**Required primitives**:
- `+`, `-`, `*`, `/`, `%` — arithmetic
- `==`, `!=`, `<`, `>`, `<=`, `>=` — comparison
- Integer literals

### Feature 5: Structured data construction

Used by MakegenOp for building JSON-like data for template rendering.

**Required primitives**:
- Object literals: `{ key: value, ... }`
- Nested construction: objects containing lists containing objects
- This is close to what the DSL already has for `@mock_response` blocks

### Feature 6: DSL-accessible data sources

Currently, pure configuration data is embedded in Rust source files and
accessed via Rust API calls. This data has no reason to live in Rust — it's
declarative configuration that belongs in DSL data files.

**Data currently hiding in Rust**:

| Data | Location | Nature |
|---|---|---|
| Clippy allowlist rules (8 rules) | `policy/pragma.rs` | Static config: crate selectors, suffix paths, rationales |
| Dead code allow rules (5 rules) | `policy/pragma.rs` | Static config: crate names, relative paths |
| Pragma allow lints (3 lints) | `policy/pragma.rs` | Static list of lint IDs |
| Crate policies (1 entry) | `policy/pragma.rs` | Static config: crate name + policy flags |
| Tool registry (12 tools) | `gunbc-tool-registry` | Static config: tool names, packages, binaries |
| Testgen specs | `gunbc-testgen-registry` | Static config: test module names, DAG paths |
| Build config | `gunbc-makegen` | Static config: cargo commands, feature flags |
| Gitignore categories (14 categories) | `gunbc-makegen` | Static config: path patterns per category |
| Codegen path templates | `codegen/ops.rs` | Static config: `target/codegen/bin`, stamp paths |
| Workspace layout | `gunbc-ir` | Derivable from DSL module structure |

**Required mechanism**:
- `data` blocks in DSL for declaring static typed configuration
- `import data from "config/pragma-policy.dag"` — DSL-to-DSL data imports
- The compiler resolves data references at compile time, not runtime

### Feature 7: Structured transport responses

The transport layer should parse command output into typed records when the
DSL declares the expected output format. This eliminates the entire category
of "parse raw text" string operations.

**Required mechanism**:
- `@parse` annotations on service calls declaring output structure
- Transport adapters that use the declared schema to parse responses
- The DSL node receives typed data, never raw strings

### What this does NOT include

Notably absent: a general-purpose string method library. No `.trim()`,
`.split()`, `.replace()`, `.strip_prefix()`, `.starts_with()`, etc. These
are symptoms of unstructured data flowing through the system. The proper fix
is structured data at the boundaries, not string manipulation in the middle.

If a future use case genuinely needs string methods (not because data arrived
unstructured, but because the domain is inherently textual), individual
methods can be added to the `String` type. But the default answer should
always be: **model the data structurally and you won't need string methods.**

### Coverage matrix: Features vs. modules

| Module | Variants | F1 Render | F2 Collections | F3 Match | F4 Arith | F5 Data | F6 Sources | F7 Transport |
|---|---|---|---|---|---|---|---|---|
| **pragma** | 3 | YES | YES | YES | - | - | YES | - |
| **makegen** | 3 | YES | YES | YES | YES | YES | YES | - |
| **bootstrap** | 4 | - | YES | YES | - | - | YES | YES |
| **codegen** | 5 | - | YES | YES | YES | - | YES | YES |
| **build** | 7 | YES | - | YES | YES | - | - | YES |

Every module is fully covered by structured modeling + these 7 features. No
module requires general-purpose string manipulation.

## Design: Phase 1 — Resolver trusts compiler (immediate, no DSL changes)

### Change 1: Default-passthrough resolver (eliminates PASSTHROUGH_CALLABLES)

**Current**: `resolve_domain()` checks custom resolvers, then
`PASSTHROUGH_CALLABLES`, then returns `unknown_callable` error.

**Proposed**: Default to passthrough for any callable the compiler validated.

```rust
fn resolve_domain(
    node_id: &str,
    module: &str,
    name: &str,
    outputs: &[Port],
    service_metadata: Option<&ServiceCallMetadata>,
) -> Result<DynOp, ResolveError> {
    // 1. Custom resolvers (modules with non-passthrough Executable impls).
    if let Some(result) = resolve_custom(node_id, module, name, outputs) {
        return result;
    }
    // 2. Service/workspace transport (generic, spec-driven).
    if module.starts_with("services.") || module.starts_with("workspace.") {
        return resolve_service_transport(node_id, module, name, service_metadata);
    }
    // 3. Resource lifecycle (generic, name-driven).
    if module == "std.resources" {
        return resolve_std_resources(name);
    }
    // 4. Default: passthrough. The compiler validated this callable exists.
    //    No list needed — if it compiled, it's resolvable.
    Ok(DynOp::new(PassthroughOp {
        output_port_names: declared_output_names(outputs),
    }))
}
```

**Why this is safe**: The DagLang compiler validates every callable reference
resolves to a declared `fn`/`func`. If `LoweredOp::Callable` reaches the
resolver, the callable exists. Passthrough is correct for any callable without
custom side-effect logic, and those are already handled by steps 1-3.

**What this eliminates**: `PASSTHROUGH_CALLABLES` (9 modules, 30+ names).
Adding a new passthrough callable requires **zero Rust changes**.

### Change 2: Inventory-based custom resolver registration (eliminates match arms)

Custom resolvers register themselves co-located with their `Executable` impls:

```rust
// In gunbc-dag/src/pragma/ops.rs:
inventory::submit!(DomainResolver {
    module: "tools.pragma",
    resolve: resolve_pragma,
});

fn resolve_pragma(node_id: &str, name: &str, outputs: &[Port])
    -> Option<Result<DynOp, ResolveError>>
{
    match name {
        "render_clippy_toml" => Some(Ok(DynOp::new(PragmaOp::RenderClippy))),
        "pragma" => Some(Ok(DynOp::new(PragmaEntrypointOp))),
        _ => None, // fall through to default passthrough
    }
}
```

**What this eliminates**: The `match module { ... }` dispatch. Adding a custom
module means adding the impl + registration in one file — `resolve.rs` never
needs editing.

Returning `None` for unrecognized callables is the key: even modules with custom
ops can have passthrough callables mixed in. No need to enumerate every callable.

### Change 3: Structural test assertions (eliminates brittle counts)

Replace `assert_eq!(dag.nodes.len(), 9)` (11+ instances) with:

```rust
assert!(spec.dag.has_node("gist.branch_resolution"));
assert!(spec.dag.is_connected());
assert!(spec.dag.has_single_sink());
```

## Design: Phase 2 — Workflows migrate to DSL (medium-term)

### Change 4: Express workflows as DSL pipelines

The 12 Rust-constructed workflow specs should become `dsl/workflows/*.dag` files,
compiled and resolved exactly like `pipelines/ci.dag` already is. This
eliminates:
- `TOOL_WORKFLOWS` registry (14 entries)
- `default_process_unit_registry()` (~80 entries)
- All `*_workflow_spec()` builder functions

### Change 5: Derive process unit claims from DSL annotations

The DSL already has `@file(READ, "{path}")` and `@file(WRITE, "{path}")`
annotations. The compiler's derivation phase (`DerivedArtifacts`) already
extracts `ResourceUsage` per node. The process unit claims can be generated
from this:

```
DSL:     @file(WRITE, "clippy.toml")
Derived: ResourceUsage { resource: "Filesystem", usage: "Write" }
Claim:   UnitClaim::write("file:workspace")
```

This closes the loop: DSL annotations -> compiler derivation -> process claims.
No Rust registry needed.

## Design: Phase 3 — DSL expression language (eliminates Rust escape hatch)

This is the core goal, not an optional long-term aspiration. Phases 1 and 2
remove registration boilerplate; Phase 3 eliminates the reason Rust is used
for business logic at all.

The approach is: **fix the data model first, then add minimal expression
support.** Most "string manipulation" disappears when data is properly
structured. What remains is legitimate rendering and collection logic.

### Change 6: Structured transport responses

Extend service call declarations so the DSL specifies the expected output
structure. The transport layer parses raw output into typed records before
the DAG node receives it. This eliminates all ad-hoc parsing (`.lines()`,
`.trim()`, `.strip_prefix()`, etc.) from business logic.

### Change 7: Structured path and glob types

Make `FilePath` a proper structured type with segments, not a string alias.
Add `GlobPattern` as a type. Path construction, joining, and pattern building
become type-safe operations. This eliminates all path string manipulation
(`.replace('\\', "/")`, `format!("{}/{}/main.rs", ...)`, `.ends_with('/')`).

### Change 8: Structured document rendering

Add `render` functions that produce typed document trees (`TextFile` /
`Document`). Sections, lines, comments, and blank lines are structural
blocks. The rendering engine handles formatting concerns (separators,
prefixes, indentation). This replaces all ad-hoc `format!()` / `write!()` /
`.push_str()` string concatenation.

### Change 9: Expression-level DSL support

Add the remaining expression features documented in "DSL Language Features
Required" above: collection operations, pattern matching, conditionals,
arithmetic, structured data construction. These are genuine computational
primitives, not string workarounds.

### Change 10: Migrate configuration data to DSL data sources

Move all static configuration currently embedded in Rust source files into
DSL data files:

```
dsl/config/pragma-policy.dag    -- clippy rules, lint policies, crate policies
dsl/config/tool-registry.dag    -- tool names, packages, binaries
dsl/config/build.dag            -- cargo commands, feature flags
dsl/config/codegen-paths.dag    -- path templates, stamp files
```

The compiler resolves these at compile time. The data is version-controlled,
diffable, and requires zero Rust knowledge to modify.

### Change 11: Migrate custom Executable impls to DSL function bodies

With structured modeling and expression support available, each custom module
migrates from Rust to DSL:

| Module | Rust ops to migrate | What eliminates them |
|---|---|---|
| `tools.infra` | Filter/count/format (5 ops) | **Delete** — already redundant with `dsl/tools/infra.dag` |
| `tools.build` | Boolean cascade + string summary (7 ops) | Conditionals + structured rendering |
| `tools.pragma` | Config rendering (3 ops) | Structured rendering + data source imports |
| `tools.bootstrap` | Shell output parsing + crate extraction (4 ops) | Structured transport responses + collection ops |
| `tools.codegen` | Path checking + manifest freshness (5 ops) | Structured paths + conditionals + data sources |
| `tools.makegen` | Registry load + JSON construction (3 ops) | Data sources + structured data literals |

After this, zero `Executable` impls exist outside the compiler/executor
infrastructure. The `resolve_custom()` path in Change 1 becomes empty.
The inventory registrations from Change 2 disappear. The resolver reduces to:

```rust
fn resolve_domain(...) -> Result<DynOp, ResolveError> {
    if module.starts_with("services.") || module.starts_with("workspace.") {
        return resolve_service_transport(...);
    }
    if module == "std.resources" {
        return resolve_std_resources(name);
    }
    // Everything is passthrough. The DSL handles all logic.
    Ok(DynOp::new(PassthroughOp { ... }))
}
```

## What stays in Rust (by design, not by escape hatch)

These are infrastructure concerns, not business logic. They don't duplicate
DSL metadata and don't grow when tools/workflows are added:

| Component | Why it's Rust | Grows when... |
|---|---|---|
| DagLang compiler | Language implementation | New DSL syntax is added |
| DAG executor | Runtime engine | New execution semantics are added |
| Transport adapters (Shell, REST, Filesystem) | System boundary / FFI | New transport protocols are added |
| Resource handle types | Capability system | New resource kinds are added |
| `WorkspaceBinary` (12 entries) | Build system (Cargo binary names) | New crate binaries are added |
| `STANDARD_SYMBOLS` (40 entries) | UI/presentation | New display symbols are added |
| `FORBIDDEN_CALLS` (guardrails) | Architectural constraint | New safety rules are added |

The key distinction: **infrastructure grows with the platform, not with the
domain.** Adding a new tool, workflow, or pipeline should never require touching
any of these.

## Implementation order

| Phase | Changes | Eliminates | Size |
|---|---|---|---|
| **1a** | Default passthrough (Change 1) | `PASSTHROUGH_CALLABLES` | S |
| **1b** | Structural tests (Change 3) | 13+ brittle count assertions | S |
| **1c** | Inventory resolvers (Change 2) | `match module` dispatch | M |
| **2a** | Workflow DSL migration (Change 4) | `TOOL_WORKFLOWS` + builder fns | L |
| **2b** | Derived claims (Change 5) | `process_registry` (80+ entries) | M |
| **3a** | Structured transport responses (Change 6) | All ad-hoc parsing (~15 string ops) | M |
| **3b** | Structured path/glob types (Change 7) | All path string manipulation (~8 string ops) | M |
| **3c** | Structured document rendering (Change 8) | All ad-hoc string concatenation (~25 string ops) | M |
| **3d** | Expression support (Change 9) | Need for Rust computation logic | L |
| **3e** | Data source migration (Change 10) | Config data in Rust files | M |
| **3f** | Custom op migration (Change 11) | All 5 custom `Executable` modules (27 ops) | L |

Phase 1a is the highest-value immediate win. Phase 3a-3c are the modeling
foundation — they eliminate most "string manipulation" by making it
unnecessary, not by supporting it. Phase 3d-3f build on that foundation
to complete the migration. The key principle: **model the data first, add
expression support for what remains.**

## Success criteria

**After Phase 1** (Rust-only, no DSL changes):
- New passthrough callable: **0 Rust files**
- New custom-behavior module: **1 file** (impl + inventory, co-located)
- New resource: **0 Rust files** (already done)

**After Phase 2** (workflow migration):
- New workflow: **1 DSL file** (no Rust)
- New process unit: **0 files** (derived from DSL annotations)

**After Phase 3** (DSL expressions + data migration):
- New tool of any complexity: **1 DSL file** (no Rust at all)
- New configuration/policy: **1 DSL data file** (no Rust at all)
- Rust code only needed for: new transport adapters, new resource handle types
- Custom `Executable` impls: **0** (down from 27 op variants across 5 modules)
- The concept of "escape to Rust" no longer exists
