# Extern Bridge Gap Analysis

Status: Active design
Goal: Eliminate the escape hatch entirely — zero extern func, zero bridge code

## Completed Work (Phases 1-4)

### Phase 1: Dead Code + Proven Identical (3 entries) — DONE

- `render_diff_markdown` — DSL body byte-identical; bridge deleted
- `prepare_scan_workspace` — dead code; deleted
- `parse_scan_result` — dead code; deleted

### Phase 2: Markdown Pure DSL Conversion (5 entries) — DONE

Created `dsl/std/markdown_render.dag` with pure DSL fn bodies for
render_code_block, render_bullet_list, render_numbered_list, render_node,
render_markdown. Types stay in `std.markdown` (types-only module).

### Phase 3: repeat() + render_heading (1 entry) — DONE

render_heading converted. RenderHeadingOp deleted.

### Phase 4: Explicit Extern Conversion (11 entries) — DONE

All remaining shadow bridges converted to honest `extern func` declarations.
compiled_fns.rs deleted. extern_impls.rs created with explicit implementations.
resolve.rs bridge lookup simplified.

## Current State (Post Phase 4)

**extern_impls.rs** contains 10 honest extern func implementations:

| # | Module | Name | What It Does |
|---|--------|------|-------------|
| 1 | std.markdown | render_tree | Recursive tree algorithm (TreeNode trie → box-drawing chars) |
| 2 | tools.gist | build_snapshot_content | Assembles markdown: heading, tree, file list, code blocks |
| 3 | tools.pragma | render_clippy_toml | Reads DISALLOWED_METHODS_ALLOWLIST_RULES from policy::pragma |
| 4 | tools.pragma | render_disallowed_methods_allowlist | Reads rules + resolves crate paths via cargo metadata |
| 5 | tools.pragma | render_pragma_lint_policy | Reads DEAD_CODE_ALLOW_RULES + PRAGMA_ALLOW_LINTS |
| 6 | tools.bootstrap | render_bootstrap_makefile | Loads ToolRegistry, delegates to makegen::render |
| 7 | tools.bootstrap | render_bootstrap_gitignore | Loads BuildConfig, delegates to makegen::gitignore |
| 8 | tools.makegen | load_registry | Calls discover_tool_defs_from_dsl() + default_core_workflows() |
| 9 | tools.makegen | render_makefile | Calls makegen::render::render_makefile(&registry) |
| 10 | tools.makegen | makegen | Entrypoint: loads registry + renders makefile end-to-end |

These are honest — the DSL declares `extern func`, the impl is visible. But
they're still escape hatches. Each one either:
- Collapses structured data to String too early (anemic modeling), or
- Accesses Rust const arrays that should be DSL data (scattered ownership)

## Diagnosis: Why These Externs Exist

### Problem 1: Anemic Modeling (entries 1-2)

Bridge functions produce String when the DSL already has structural types to
carry the semantics. They format data into text instead of returning typed
intermediate representations that DSL renderers could handle.

**render_tree**: The Rust impl builds a `TreeNode { children: BTreeMap }` trie
then renders with `├──`, `└──`, `│` characters via recursive DFS. But the DSL
already has box_draw.dag with character vocabularies and `repeat_char` via fold.
The algorithm can be modeled structurally.

**build_snapshot_content**: The Rust impl does `format!()` to assemble markdown
with headings, tree, file list, code blocks. Every section maps 1:1 to
MarkdownNode variants that already exist in std.markdown.

### Problem 2: Scattered Data Ownership (entries 3-10)

Eight externs access hardcoded Rust const arrays or registry functions:

| Rust Data | Size | Nature |
|-----------|------|--------|
| DISALLOWED_METHODS_ALLOWLIST_RULES | 13 rules | Static const, all fields known at repo-design time |
| DEAD_CODE_ALLOW_RULES | 5 rules | Static const |
| PRAGMA_ALLOW_LINTS | 4 entries | Static const |
| default_core_workflows() | 20 specs | Static const |
| default_meta_targets() | 8 targets | Static const |
| discover_tool_defs_from_dsl() | dynamic | Already reads DSL files — could feed back as artifact |
| workspace_layout_or_none() | dynamic | Calls cargo metadata, but fallback_pattern already hardcodes the paths |

Every "dynamic" call has a hardcoded fallback that IS the data. The discovery
is vestigial — crate paths are stable repo conventions. This data belongs in
DSL data declarations, following the compositional modeling philosophy: each
concern modeled once, compiler composes all layers.

## New Phases: Escape Hatch Elimination

### Principle: One Representation (Lane A)

Each concern has exactly one canonical source in DSL. No parallel truth between
DSL and Rust. The Rust substrate compiles and executes — it has no knowledge of
markdown, makefiles, pragma policies, or workspace structure.

### Phase ordering

Phases are numbered by conceptual grouping, not execution order. The critical
path depends on compiler feature availability:

- **Phase 6 can start immediately** — uses only existing DSL features (match,
  filter, map, starts_with, data declarations). flat_map infra exists but needs
  validation. Eliminates 3 externs (entries 3-5) with zero compiler work.
- **Phase 7a-7c (data + rendering) can start immediately** — data declarations
  and map/join are proven. Only 7b (compiler artifact) requires compiler-side
  work. Eliminates 5 externs (entries 6-10).
- **Phase 5 is blocked on compiler features** — requires recursive types,
  recursive functions, group_by, enumerate, split, skip. This is the largest
  compiler investment. Eliminates 2 externs (entries 1-2).

Recommended execution: 6 → 7 → 5 → 8 (or 6 and 7 in parallel).

### Phase 5: Structural Modeling — Anemic Entries (entries 1-2)

**Prerequisite**: 6 compiler features must be added first (see "Compiler features
needed" below). Recommended: build compiler features incrementally, validating
each with a small DSL test before attempting the full tree rendering migration.

Lift return types from String to structured data. Introduce types that carry
semantics the bridge functions were collapsing.

#### 5a. Tree rendering types + recursive functions

Model the tree as three layers:

```
type TreeGlyph = Pipe | Space | Branch | Corner

type TreeLine {
  prefix: List<TreeGlyph>
  connector: TreeGlyph
  name: String
}

type DirEntry {
  name: String
  children: List<DirEntry>   // recursive type
}
```

Tree character data connects to box_draw.dag vocabulary:

```
data unicode_tree_chars: TreeChars = {
  pipe: "│   ", space: "    ", branch: "├── ", corner: "└── "
}

data ascii_tree_chars: TreeChars = {
  pipe: "|   ", space: "    ", branch: "|-- ", corner: "`-- "
}
```

Rendering is pure DSL (match + map + join):

```
fn render_glyph(g: TreeGlyph) -> String {
  match g { Pipe => tree_chars.pipe, Space => tree_chars.space, ... }
}

fn render_tree_line(line: TreeLine) -> String {
  let prefix = line.prefix |> map(g => render_glyph(g: g)) |> join("")
  "{prefix}{render_glyph(g: line.connector)}{line.name}"
}

fn render_tree(lines: List<TreeLine>) -> String {
  let body = lines |> map(l => render_tree_line(line: l)) |> join("\n")
  "```\n.\n{body}\n```"
}
```

Algorithm uses recursive functions:

```
fn build_dir_entries(paths: List<String>) -> List<DirEntry> {
  paths |> map(p => p |> split("/"))
    |> group_by(parts => parts |> first())
    |> map_entries((name, groups) => {
      let sub = groups |> filter(g => g |> count() > 1)
        |> map(g => g |> skip(1) |> join("/"))
      DirEntry { name: name, children: build_dir_entries(paths: sub) }
    })
    |> sort_by(e => e.name)
}

fn flatten_entries(entries: List<DirEntry>, prefix: List<TreeGlyph>) -> List<TreeLine> {
  let total = entries |> count()
  entries |> enumerate() |> flat_map((i, entry) => {
    let is_last = i == total - 1
    let connector = if is_last { Corner } else { Branch }
    let line = TreeLine { prefix: prefix, connector: connector, name: entry.name }
    let child_prefix = prefix |> append([if is_last { Space } else { Pipe }])
    [line] |> append(flatten_entries(entries: entry.children, prefix: child_prefix))
  })
}
```

Delete RenderTreeOp + extern func render_tree declaration.

DSL features required: recursive types, recursive functions, group_by,
enumerate, flat_map, split, first, skip.

#### 5b. build_snapshot_content as MarkdownDoc constructor

Every section maps to existing MarkdownNode variants:

```
Heading { level: H1, text: "Workspace Snapshot" }
Paragraph { text: "Branch: `{branch}`" }
Heading { level: H2, text: "Directory Tree" }
Tree { paths: sorted_files }
Heading { level: H2, text: "Skipped Entries" }     // conditional
BulletList { items: skipped }                        // conditional
Heading { level: H2, text: "File Contents" }
  for each (path, content) in zip(files, file_contents):
    Heading { level: H3, text: path }
    CodeBlock { language: lang_for_path(path), code: content }
```

Function becomes pure DSL node assembly. render_markdown() handles the rest.

Add lang_for_path as data declaration in languages.dag (26 static entries
mapping file extension to syntax hint for code fence highlighting).

Delete BuildSnapshotContentOp.

DSL features required: conditional list (if/else in list context), zip.

### Phase 6: Workspace Model + Policy Migration (entries 3-5)

**No missing compiler features.** All required DSL features are proven in
existing .dag files (filter with field access + equality, starts_with, match on
sum types, data declarations with nested structs). One risk: flat_map has lowerer
infrastructure (CollectionOpKind::FlatMap) but zero DSL usage — validate with a
simple test case before 6c.

Move policy data and workspace knowledge from Rust const arrays to DSL data
declarations. This is Lane A work — establishing DSL as the single source of
truth for workspace self-understanding.

#### 6a. Workspace crate model

Create `dsl/config/workspace.dag`:

```
type CrateTier = Foundation | Core | Application

type CrateSpec {
  name: String
  path: String
  tier: CrateTier
  is_producer: Bool
}

data workspace_crates: List<CrateSpec> = [
  { name: "gunbc-infra",         path: "core/infra",           tier: Foundation, is_producer: false },
  { name: "gunbc-ir",            path: "core/ir",              tier: Core,       is_producer: false },
  { name: "gunbc-exec",          path: "core/exec",            tier: Core,       is_producer: false },
  { name: "gunbc-codegen",       path: "core/codegen",         tier: Core,       is_producer: true  },
  { name: "gunbc-dag",           path: "gunbc-dag",            tier: Application, is_producer: true },
  { name: "gunbc-lib-transport", path: "lib/transport",        tier: Application, is_producer: false },
  { name: "gunbc-lib-primitives",path: "lib/primitives",       tier: Application, is_producer: false },
  { name: "gunbc-lib-gcp-ops",   path: "lib/gcp-ops",         tier: Application, is_producer: false },
  { name: "daglang-syntax",      path: "core/daglang/daglang-syntax", tier: Core, is_producer: false },
  { name: "daglang-lower",       path: "core/daglang/daglang-lower",  tier: Core, is_producer: false },
  { name: "daglang-derive",      path: "core/daglang/daglang-derive", tier: Core, is_producer: false },
  { name: "daglang-driver",      path: "core/daglang/daglang-driver", tier: Core, is_producer: false },
  { name: "daglang-cli",         path: "core/daglang/daglang-cli",    tier: Core, is_producer: false },
  // ... remaining crates
]
```

CI test validates workspace_crates agrees with Cargo.toml workspace members.
Replaces cargo metadata calls and all fallback_pattern strings in pragma.rs.

#### 6b. Structured policy rules

Extend `dsl/config/pragma_policy.dag` with full rule structures lifted from
pragma.rs const arrays:

```
type CrateSelector = Exact { name: String } | Prefix { prefix: String }

type AllowlistRule {
  selector: CrateSelector
  suffix: String
  as_prefix: Bool
  rationale: String
}

data allowlist_rules: List<AllowlistRule> = [
  { selector: Exact { name: "gunbc-lib-transport" }, suffix: "",
    as_prefix: true, rationale: "transport boundary: std::process allowed" },
  { selector: Prefix { prefix: "daglang-" }, suffix: "",
    as_prefix: true, rationale: "DSL compiler crates: file I/O for .dag parsing" },
  // ... 11 more rules, verbatim from DISALLOWED_METHODS_ALLOWLIST_RULES
]

data dead_code_rules: List<DeadCodeRule> = [
  { crate_name: "gunbc-dag", relative_path: "src/makegen/registry.rs" },
  { crate_name: "gunbc-lib-gcp-ops", relative_path: "src/graph.rs" },
  // ... 3 more
]

data allow_lints: List<String> = [
  "clippy::large_enum_variant", "clippy::too_many_arguments",
  "clippy::vec_init_then_push", "unused_variables"
]
```

#### 6c. Policy resolution + rendering as pure DSL

Crate path resolution becomes a DSL function over the workspace model:

```
fn resolve_crate_path(selector: CrateSelector, crates: List<CrateSpec>) -> List<String> {
  match selector {
    Exact { name } => crates |> filter(c => c.name == name) |> map(c => c.path)
    Prefix { prefix } => crates |> filter(c => c.name |> starts_with(prefix)) |> map(c => c.path)
  }
}
```

Pragma render functions use Document type (already supported via dag_util):

```
fn render_allowlist_doc() -> Document {
  doc_with_header(
    header: generated_header(tool: "pragma"),
    comment_prefix: "#",
    sections: allowlist_rules |> flat_map(rule =>
      resolve_allowlist_pattern(rule: rule, crates: workspace_crates)
        |> map(pattern => section(lines: [
          comment_line(text: rule.rationale),
          text_line(text: pattern)
        ]))
    )
  )
}
```

Delete all pragma rendering from policy/pragma.rs. Delete extern func
declarations and all shadow fn bodies from pragma.dag.

### Phase 7: Registry Migration (entries 6-10)

**7a and 7c need no new compiler features.** Data declarations, map, join,
match on null, `??` are all proven. flat_map (for tool target expansion) needs
validation (same as Phase 6). Only 7b requires compiler-side work (artifact
emitter).

Move registry constants from Rust to DSL. Establish compiler artifact
feedback loop for tool discovery.

#### 7a. Workflow and target data declarations

Create `dsl/config/build_workflows.dag`:

```
type WorkflowSpec {
  name: String
  description: String
  deps: List<String>
  resources: List<String>
}

data core_workflows: List<WorkflowSpec> = [
  { name: "preflight-fix", description: "Auto-fix formatting and lint issues",
    deps: [], resources: [] },
  { name: "ensure-codegen", description: "Ensure generated code is fresh",
    deps: ["preflight-fix"], resources: ["codegen"] },
  { name: "build", description: "Build all targets",
    deps: ["ensure-codegen"], resources: [] },
  { name: "test", description: "Run test suite",
    deps: ["build"], resources: [] },
  // ... 16 more, lifted from registry.rs default_core_workflows()
]

type MetaTarget {
  name: String
  description: String
  has_check: Bool
  has_fix: Bool
  resources: List<String>
}

data meta_targets: List<MetaTarget> = [
  { name: "test",    description: "Run test suite",   has_check: false, has_fix: false, resources: [] },
  { name: "clippy",  description: "Lint check",       has_check: true,  has_fix: true,  resources: [] },
  { name: "fmt",     description: "Format check",     has_check: true,  has_fix: true,  resources: [] },
  // ... 5 more
]
```

Delete default_core_workflows() and default_meta_targets() from registry.rs.

#### 7b. Tool registry as compiler artifact

DSL compiler emits `dsl/generated/tool_registry.dag` when compiling
`dsl/tools/*.dag`:

```
data discovered_tools: List<ToolInfo> = [
  { name: "pragma", short_name: "pragma",
    outputs: ["clippy.toml", "tools/disallowed-methods-allowlist.txt", "tools/pragma-lint-policy.txt"],
    entrypoints: [{ name: "pragma", params: [...] }] },
  { name: "bootstrap", short_name: "bootstrap",
    outputs: ["Makefile", ".gitignore"],
    entrypoints: [{ name: "bootstrap", params: [...] }] },
  // ... auto-discovered
]
```

Committed like other seed files (COMMITTED_SEED_FILES). Drift detection test
validates freshness. Replaces runtime discover_tool_defs_from_dsl() in
LoadRegistryOp.

#### 7c. Makefile / gitignore rendering as DSL

New DSL types mirror existing Rust render IR:

```
type MakefileTarget {
  name: String
  deps: List<String>
  body: List<String>
  comment: String?
}

type GitignoreCategory {
  name: String
  source: String?
  items: List<String>
  rationale: String?
}
```

Rendering is trivial DSL (the Rust renderers are join-with-delimiters):

```
fn render_target(t: MakefileTarget) -> String {
  let comment_line = match t.comment { null => "", c => "# {c}\n" }
  let deps_str = t.deps |> join(" ")
  let body_str = t.body |> map(line => "\t{line}") |> join("\n")
  "{comment_line}{t.name}: {deps_str}\n{body_str}\n"
}

fn render_category(c: GitignoreCategory) -> String {
  let source = c.source ?? "unknown"
  let header = "# --- {c.name} (from {source}) ---"
  let rationale = match c.rationale { null => "", r => "# {r}\n" }
  let items = c.items |> join("\n")
  "{header}\n{rationale}{items}\n"
}
```

Makefile assembly imports workflow + tool data:

```
import config.build_workflows { core_workflows, meta_targets }
import generated.tool_registry { discovered_tools }

fn build_makefile_targets() -> List<MakefileTarget> {
  let workflow_targets = core_workflows |> map(w =>
    MakefileTarget { name: w.name, deps: w.deps, body: [...], comment: w.description }
  )
  let tool_targets = discovered_tools |> flat_map(t => [
    MakefileTarget { name: t.short_name, deps: ["ensure-codegen"], body: [...], comment: null },
    MakefileTarget { name: "{t.short_name}-dry", deps: [], body: [...], comment: "Dry run" }
  ])
  workflow_targets |> append(tool_targets)
}
```

Delete LoadRegistryOp, RenderMakefileCompiledOp, MakegenEntrypointOp,
GenerateBootstrapMakefileOp, GenerateBootstrapGitignoreOp. Delete extern func
declarations from makegen.dag and bootstrap.dag.

### Phase 8: Delete extern_impls.rs

After all 10 entries are pure DSL:
- Delete gunbc-dag/src/extern_impls.rs
- Remove mod extern_impls from gunbc-dag/src/lib.rs
- Delete resolve_extern_call() from resolve.rs (no extern func declarations exist)
- Remove all_extern_symbols() and lookup_extern_impl()
- Update tool_registration.rs: remove extern symbol validation, update
  ALLOWED_PASSTHROUGH_CALLABLES for all conversions
- Verify: zero `extern func` declarations in any .dag file

## DSL Features Required

### Already available

| Feature | Evidence | Used By |
|---------|----------|---------|
| first() | builtin_callable_contracts, arity 0 | Phase 5a path grouping |
| starts_with() | builtin_callable_contracts, arity 1 | Phase 6c crate selector |
| flat_map() | CollectionOpKind::FlatMap in lowerer | Phase 5a, 6c (no DSL usage yet — needs validation) |
| filter() on List | collection_op_kind recognizes "filter" | Phase 6c crate filtering (List only, not Map) |
| map/join/fold/count | Proven across box_draw.dag, markdown_render.dag, etc. | All phases |
| match on sum types | Proven in box_draw.dag, markdown_render.dag | All phases |
| if/else expressions | Proven in box_draw.dag repeat_char | Phase 5a, 5b |
| data declarations | Proven in languages.dag (complex nested structs) | Phase 6, 7 |
| ?? (null coalesce) | Proven in markdown_render.dag (`language ?? ""`) | Phase 7c optional fields |
| match on null | Proven in box_draw.dag (`match x { null => ..., c => ... }`) | Phase 7c optional fields |

### Compiler features needed

Ordered by how many entries they unblock:

| Feature | Status | Phase | Entries Unblocked | Also Useful For |
|---------|--------|-------|-------------------|-----------------|
| Recursive types | MISSING | 5 | render_tree | AST modeling, nested configs |
| Recursive functions | MISSING | 5 | render_tree, build_snapshot_content | Tree/graph traversal, JSON |
| group_by(key_fn) | MISSING | 5 | render_tree (path grouping) | Data aggregation, reports |
| enumerate() | MISSING | 5 | render_tree (is_last), snapshot | Numbered lists, indexing |
| split(delim) | MISSING | 5 | render_tree (path segments) | Config parsing, CSV |
| skip(n) | MISSING | 5 | render_tree (path subpath) | Subsequence ops |
| zip() | MISSING | 5 | build_snapshot_content | Parallel iteration |
| Conditional list | MISSING | 5 | build_snapshot_content, pragma | Conditional assembly |
| Compiler registry artifact | MISSING | 7 | load_registry elimination | Build system |

### Feature dependency chain

Phase 5a (tree rendering) is the critical path — it requires 6 missing features.
Phase 5b (snapshot content) adds 2 more (zip, conditional list).
Phase 6 needs zero new features (List filter + starts_with are proven; flat_map
needs validation only). Phase 7 needs a compiler-side artifact emitter (7b).

Recommended build order for compiler features:
1. split, skip (string/list primitives — small, testable independently; first already exists)
2. enumerate, flat_map validation (collection ops — flat_map infra exists)
3. group_by (collection op — harder, needs key function + grouping semantics)
4. Recursive types + recursive functions (largest feature — type system + lowerer)
5. Conditional list assembly (list context if/else — may be expressible via flat_map)
6. zip (parallel iteration — independent of recursion)
7. Compiler registry artifact (tooling, not language feature)

## Endstate

```
compiled_fns.rs    — deleted (Phase 4)
extern_impls.rs    — deleted (Phase 8)
extern func        — zero declarations in any .dag file
lookup_compiled_fn — deleted (Phase 4)
lookup_extern_impl — deleted (Phase 8)

resolve.rs:
  - resolve_domain() → DeclaredOutputCallableOp for ALL callables
  - resolve_extern_call() — deleted
  - Bridge lookup step — deleted
```

Where data lives:

| Data | Location |
|------|----------|
| Workspace crate map | config/workspace.dag |
| Policy rules | config/pragma_policy.dag |
| Build workflows | config/build_workflows.dag |
| Meta targets | config/build_workflows.dag |
| Tool registry | generated/tool_registry.dag (compiler artifact) |
| Tree characters | std/box_draw.dag (extended) |
| Heading levels | std/markdown.dag (HeadingLevel sum type) |
| Language hints | std/languages.dag (extended) |

Where algorithms live:

| Algorithm | Location |
|-----------|----------|
| Tree rendering | std/markdown_render.dag (recursive DirEntry + TreeLine) |
| Crate path resolution | config/pragma_policy.dag (match + filter) |
| Makefile assembly | tools/makegen.dag (map over workflow + tool data) |
| Document rendering | shared/dag_util.dag (already pure DSL) |
| Markdown rendering | std/markdown_render.dag (already pure DSL) |

What Rust does:

1. Compiles DSL to DAG (the compiler)
2. Executes DAG nodes (the runtime)
3. Emits tool_registry.dag as compiler artifact

It has no knowledge of markdown, makefiles, pragma policies, tree rendering,
crate tiers, or workflow dependencies. All domain logic is DSL.

## Alignment

This is a Lane A ("One Representation") outcome:
- Policy rules: one place (pragma_policy.dag)
- Workspace structure: one place (workspace.dag)
- Tool registry: one place (generated artifact)
- No shadow bridges, no silent overrides, no parallel truth

It enables Lane B ("Proven Correct"):
- CI validates workspace.dag agrees with Cargo.toml
- CI validates tool_registry.dag is fresh
- Compiler validates policy rules reference valid crate specs
- Every fn body executes — no silent overrides possible

## Risk Mitigations

| Risk | Mitigation |
|------|------------|
| DSL fn body SubDag execution fails | Phases 1-4 validated pattern with 8+ entries |
| Recursive functions are a large DSL feature | Tree rendering is the forcing function; feature pays off broadly (AST, JSON, nested configs) |
| Workspace crate paths change | CI test validates workspace.dag vs Cargo.toml; single update point |
| Generated tool_registry.dag drifts | Existing drift detection pattern (COMMITTED_SEED_FILES) |
| Data declarations don't lower to runtime values | languages.dag data declarations prove the pattern |
| Conditional list assembly not yet in DSL | build_snapshot_content can return MarkdownDoc from extern as interim; conditional list unblocks full migration |
| Makefile workflow bodies are complex (20+ cases) | Workflow body data lives in build_workflows.dag data declarations alongside the specs; rendering is trivial join-with-delimiters |
